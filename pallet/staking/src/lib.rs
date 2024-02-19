// This file is part of Darwinia.
//
// Copyright (C) 2018-2023 Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

//! # Darwinia parachain staking pallet
//!
//! ## Overview
//!
//! This is a completely specialized stake pallet designed only for Darwinia parachain.
//! So, this pallet will eliminate the generic parameters as much as possible.
//!
//! ### Acceptable stakes:
//! - RING: Darwinia's native token
//! - Deposit: Locking RINGs' ticket

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
#![deny(unused_crate_dependencies)]

pub mod migration;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;
pub use weights::WeightInfo;

pub use darwinia_staking_traits::*;

// crates.io
use codec::FullCodec;
// darwinia
use dc_types::{Balance, Moment};
// substrate
use frame_support::{
	pallet_prelude::*, traits::Currency, DefaultNoBound, EqNoBound, PalletId, PartialEqNoBound,
};
use frame_system::{pallet_prelude::*, RawOrigin};
use sp_runtime::{
	traits::{AccountIdConversion, Convert, One, Zero},
	Perbill,
};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

/// Make it easier to call a function on a specific exposure storage.
#[macro_export]
macro_rules! call_on_exposure {
	($s_e:expr, <$s:ident<$t:ident>>$($f:tt)*) => {{
		match $s_e {
			($crate::ExposureCacheState::$s, _, _) => Ok(<$crate::ExposureCache0<$t>>$($f)*),
			(_, $crate::ExposureCacheState::$s, _) => Ok(<$crate::ExposureCache1<$t>>$($f)*),
			(_, _, $crate::ExposureCacheState::$s) => Ok(<$crate::ExposureCache2<$t>>$($f)*),
			_ => {
				log::error!("[pallet::staking] exposure cache states must be correct; qed");

				Err("[pallet::staking] exposure cache states must be correct; qed")
			},
		}
	}};
	(<$s:ident<$t:ident>>$($f:tt)*) => {{
		let s = <$crate::ExposureCacheStates<$t>>::get();

		$crate::call_on_exposure!(s, <$s<$t>>$($f)*)
	}};
}

#[frame_support::pallet]
pub mod pallet {
	// darwinia
	use crate::*;

	// TODO: limit the number of nominators that a collator can have.
	// const MAX_NOMINATIONS: u32 = 32;

	// Deposit helper for runtime benchmark.
	#[cfg(feature = "runtime-benchmarks")]
	use darwinia_deposit::Config as DepositConfig;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

	/// Empty trait acts as a place holder to satisfy the `#[pallet::config]` macro.
	#[cfg(not(feature = "runtime-benchmarks"))]
	pub trait DepositConfig {}

	#[pallet::config]
	pub trait Config: frame_system::Config + DepositConfig {
		/// Override the [`frame_system::Config::RuntimeEvent`].
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// RING [`Stake`] interface.
		type Ring: Stake<AccountId = Self::AccountId, Item = Balance>;

		// TODO: Remove.
		/// KTON [`Stake`] interface.
		type Kton: Stake<AccountId = Self::AccountId, Item = Balance>;

		/// Deposit [`StakeExt`] interface.
		type Deposit: StakeExt<AccountId = Self::AccountId, Amount = Balance>;

		/// Currency interface to pay the reward.
		type Currency: Currency<Self::AccountId>;

		/// Inflation and reward manager.
		type IssuingManager: IssuingManager<Self>;

		/// Pass [`pallet_session::Config::ShouldEndSession`]'s result to here.
		type ShouldEndSession: Get<bool>;

		/// Minimum time to stake at least.
		#[pallet::constant]
		type MinStakingDuration: Get<BlockNumberFor<Self>>;

		/// Maximum deposit count.
		#[pallet::constant]
		type MaxDeposits: Get<u32>;

		/// Maximum unstaking/unbonding count.
		#[pallet::constant]
		type MaxUnstakings: Get<u32>;

		/// KTON staker contract address.
		#[pallet::constant]
		type KtonStaker: Get<Self::AccountId>;
	}

	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An account has staked some assets.
		Staked { who: T::AccountId, ring_amount: Balance, deposits: Vec<DepositId<T>> },
		/// An account has unstaked assets.
		Unstaked { who: T::AccountId, ring_amount: Balance, deposits: Vec<DepositId<T>> },
		/// A collator has updated their commission.
		CommissionUpdated { who: T::AccountId, commission: Perbill },
		/// A payout has been made for the staker.
		Payout { who: T::AccountId, amount: Balance },
		/// Unable to pay the staker's reward.
		Unpaid { who: T::AccountId, amount: Balance },
		/// A new collator set has been elected.
		Elected { collators: Vec<T::AccountId> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Exceed maximum deposit count.
		ExceedMaxDeposits,
		/// Exceed maximum unstaking/unbonding count.
		ExceedMaxUnstakings,
		/// Deposit not found.
		DepositNotFound,
		/// You are not a staker.
		NotStaker,
		/// Target is not a collator.
		TargetNotCollator,
		/// Collator count mustn't be zero.
		ZeroCollatorCount,
		/// No reward to pay for this collator.
		NoReward,
	}

	/// All staking ledgers.
	#[pallet::storage]
	#[pallet::getter(fn ledger_of)]
	pub type Ledgers<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Ledger<T>>;

	/// The map from (wannabe) collator to the preferences of that collator.
	#[pallet::storage]
	#[pallet::getter(fn collator_of)]
	pub type Collators<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Perbill>;

	/// Exposure cache states.
	///
	/// To avoid extra DB RWs during new session, such as:
	/// ```nocompile
	/// previous = current;
	/// current = next;
	/// next = elect();
	/// ```
	///
	/// Now, with data:
	/// ```nocompile
	/// cache1 == previous;
	/// cache2 == current;
	/// cache3 == next;
	/// ```
	/// Just need to shift the marker and write the storage map once:
	/// ```nocompile
	/// mark(cache3, current);
	/// mark(cache2, previous);
	/// mark(cache1, next);
	/// cache1 = elect();
	/// ```
	#[pallet::storage]
	#[pallet::getter(fn exposure_cache_states)]
	pub type ExposureCacheStates<T: Config> = StorageValue<
		_,
		(ExposureCacheState, ExposureCacheState, ExposureCacheState),
		ValueQuery,
		ExposureCacheStatesDefault<T>,
	>;
	/// Default value for [`ExposureCacheStates`].
	#[pallet::type_value]
	pub fn ExposureCacheStatesDefault<T: Config>(
	) -> (ExposureCacheState, ExposureCacheState, ExposureCacheState) {
		(ExposureCacheState::Previous, ExposureCacheState::Current, ExposureCacheState::Next)
	}

	/// Exposure cache 0.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn exposure_cache_0_of)]
	pub type ExposureCache0<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Exposure<T::AccountId>>;

	/// Exposure cache 1.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn exposure_cache_1_of)]
	pub type ExposureCache1<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Exposure<T::AccountId>>;

	/// Exposure cache 2.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn exposure_cache_2_of)]
	pub type ExposureCache2<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Exposure<T::AccountId>>;

	/// The ideal number of active collators.
	#[pallet::storage]
	#[pallet::getter(fn collator_count)]
	pub type CollatorCount<T> = StorageValue<_, u32, ValueQuery>;

	/// The map from nominator to their nomination preferences, namely the collator that
	/// they wish to support.
	#[pallet::storage]
	#[pallet::getter(fn nominator_of)]
	pub type Nominators<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, T::AccountId>;

	/// Number of blocks authored by the collator within current session.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn authored_block_count)]
	pub type AuthoredBlocksCount<T: Config> =
		StorageValue<_, (BlockNumberFor<T>, BTreeMap<T::AccountId, BlockNumberFor<T>>), ValueQuery>;

	/// All outstanding rewards since the last payment.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn pending_reward_of)]
	pub type PendingRewards<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Balance>;

	/// Active session's start-time.
	#[pallet::storage]
	#[pallet::getter(fn session_start_time)]
	pub type SessionStartTime<T: Config> = StorageValue<_, Moment, ValueQuery>;

	/// Elapsed time.
	#[pallet::storage]
	#[pallet::getter(fn elapsed_time)]
	pub type ElapsedTime<T: Config> = StorageValue<_, Moment, ValueQuery>;

	#[derive(DefaultNoBound)]
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// Current timestamp.
		pub now: Moment,
		/// The running time of Darwinia1.
		pub elapsed_time: Moment,
		/// Genesis collator count.
		pub collator_count: u32,
		/// Genesis collator preferences.
		pub collators: Vec<(T::AccountId, Balance)>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			if self.collator_count == 0 {
				panic!("[pallet::staking] collator count mustn't be 0");
			}

			<SessionStartTime<T>>::put(self.now);
			<ElapsedTime<T>>::put(self.elapsed_time);
			<CollatorCount<T>>::put(self.collator_count);

			self.collators.iter().for_each(|(who, ring_amount)| {
				<Pallet<T>>::stake(
					RawOrigin::Signed(who.to_owned()).into(),
					*ring_amount,
					Vec::new(),
				)
				.expect("[pallet::staking] 0, genesis must be built; qed");
				<Pallet<T>>::collect(RawOrigin::Signed(who.to_owned()).into(), Default::default())
					.expect("[pallet::staking] 1, genesis must be built; qed");
				<Pallet<T>>::nominate(RawOrigin::Signed(who.to_owned()).into(), who.to_owned())
					.expect("[pallet::staking] 2, genesis must be built; qed");
			});
		}
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_: BlockNumberFor<T>) -> Weight {
			// There are already plenty of tasks to handle during the new session,
			// so refrain from assigning any additional ones here.
			if !T::ShouldEndSession::get() {
				call_on_exposure!(<Previous<T>>::iter_keys()
					// TODO?: make this value adjustable
					.take(1)
					.fold(Zero::zero(), |acc, e| acc
						+ Self::payout_inner(e).unwrap_or(Zero::zero())))
				.unwrap_or_default()
			} else {
				Zero::zero()
			}
		}
	}
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add stakes to the staking pool.
		///
		/// This will transfer the stakes to a pallet/contact account.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::stake(deposits.len() as _))]
		pub fn stake(
			origin: OriginFor<T>,
			ring_amount: Balance,
			deposits: Vec<DepositId<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			if ring_amount == 0 && deposits.is_empty() {
				return Ok(());
			}

			<Ledgers<T>>::try_mutate(&who, |l| {
				let l = if let Some(l) = l {
					l
				} else {
					<frame_system::Pallet<T>>::inc_consumers(&who)?;

					*l = Some(Ledger {
						staked_ring: Default::default(),
						staked_deposits: Default::default(),
						unstaking_ring: Default::default(),
						unstaking_deposits: Default::default(),
					});

					l.as_mut().expect("[pallet::staking] `l` must be some; qed")
				};

				if ring_amount != 0 {
					Self::stake_ring(&who, &mut l.staked_ring, ring_amount)?;
				}

				for d in deposits.clone() {
					Self::stake_deposit(&who, l, d)?;
				}

				DispatchResult::Ok(())
			})?;

			Self::deposit_event(Event::Staked { who, ring_amount, deposits });

			Ok(())
		}

		/// Withdraw stakes from the staking pool.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::unstake(deposits.len() as _))]
		pub fn unstake(
			origin: OriginFor<T>,
			ring_amount: Balance,
			deposits: Vec<DepositId<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			if ring_amount == 0 && deposits.is_empty() {
				return Ok(());
			}

			<Ledgers<T>>::try_mutate(&who, |l| {
				let l = l.as_mut().ok_or(<Error<T>>::NotStaker)?;

				if ring_amount != 0 {
					Self::unstake_ring(&mut l.staked_ring, &mut l.unstaking_ring, ring_amount)?;
				}

				for d in deposits {
					Self::unstake_deposit(l, d)?;
				}

				DispatchResult::Ok(())
			})?;

			Ok(())
		}

		/// Cancel the `unstake` operation.
		///
		/// Re-stake the unstaking assets immediately.
		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::restake(deposits.len() as _))]
		pub fn restake(
			origin: OriginFor<T>,
			ring_amount: Balance,
			deposits: Vec<DepositId<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			if ring_amount == 0 && deposits.is_empty() {
				return Ok(());
			}

			<Ledgers<T>>::try_mutate(&who, |l| {
				let l = l.as_mut().ok_or(<Error<T>>::NotStaker)?;

				if ring_amount != 0 {
					Self::restake_ring(&mut l.staked_ring, &mut l.unstaking_ring, ring_amount)?;
				}

				for d in deposits {
					Self::restake_deposit(l, d)?;
				}

				DispatchResult::Ok(())
			})?;

			Ok(())
		}

		/// Claim the stakes from the pallet/contract account.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::claim())]
		pub fn claim(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Deposit doesn't need to be claimed.
			Self::claim_unstakings(&who)?;
			Self::try_clean_ledger_of(&who);

			Ok(())
		}

		/// Declare the desire to collect.
		///
		/// Effects will be felt at the beginning of the next session.
		#[pallet::call_index(4)]
		#[pallet::weight(<T as Config>::WeightInfo::collect())]
		pub fn collect(origin: OriginFor<T>, commission: Perbill) -> DispatchResult {
			let who = ensure_signed(origin)?;

			<Collators<T>>::mutate(&who, |c| *c = Some(commission));

			Self::deposit_event(Event::CommissionUpdated { who, commission });

			Ok(())
		}

		/// Declare the desire to nominate a collator.
		///
		/// Effects will be felt at the beginning of the next session.
		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::nominate())]
		pub fn nominate(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			if !<Ledgers<T>>::contains_key(&who) {
				Err(<Error<T>>::NotStaker)?
			}
			if !<Collators<T>>::contains_key(&target) {
				Err(<Error<T>>::TargetNotCollator)?;
			}

			<Nominators<T>>::mutate(&who, |n| *n = Some(target));

			Ok(())
		}

		/// Declare no desire to either collect or nominate.
		///
		/// Effects will be felt at the beginning of the next era.
		///
		/// If the target is a collator, its nominators need to re-nominate.
		#[pallet::call_index(6)]
		#[pallet::weight(<T as Config>::WeightInfo::chill())]
		pub fn chill(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			<Collators<T>>::remove(&who);
			<Nominators<T>>::remove(&who);

			Ok(())
		}

		/// Making the payout for the specified collators and its nominators.
		#[pallet::call_index(8)]
		#[pallet::weight(<T as Config>::WeightInfo::payout())]
		pub fn payout(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			ensure_signed(origin)?;

			Self::payout_inner(who)?;

			Ok(())
		}

		/// Set collator count.
		///
		/// This will apply to the incoming session.
		///
		/// Require root origin.
		#[pallet::call_index(7)]
		#[pallet::weight(<T as Config>::WeightInfo::set_collator_count())]
		pub fn set_collator_count(origin: OriginFor<T>, count: u32) -> DispatchResult {
			ensure_root(origin)?;

			if count == 0 {
				return Err(<Error<T>>::ZeroCollatorCount)?;
			}

			<CollatorCount<T>>::put(count);

			Ok(())
		}
	}
	impl<T> Pallet<T>
	where
		T: Config,
	{
		fn stake_ring(who: &T::AccountId, staked: &mut Balance, amount: Balance) -> DispatchResult {
			<T as Config>::Ring::stake(who, amount)?;

			*staked = staked
				.checked_add(amount)
				.ok_or("[pallet::staking] `u128` must not be overflowed; qed")?;

			Ok(())
		}

		fn stake_deposit(
			who: &T::AccountId,
			ledger: &mut Ledger<T>,
			deposit: DepositId<T>,
		) -> DispatchResult {
			T::Deposit::stake(who, deposit)?;

			ledger.staked_deposits.try_push(deposit).map_err(|_| <Error<T>>::ExceedMaxDeposits)?;

			Ok(())
		}

		fn unstake_ring(
			staked: &mut Balance,
			unstaking: &mut BoundedVec<(Balance, BlockNumberFor<T>), T::MaxUnstakings>,
			amount: Balance,
		) -> DispatchResult {
			*staked = staked
				.checked_sub(amount)
				.ok_or("[pallet::staking] `u128` must not be overflowed; qed")?;

			unstaking
				.try_push((
					amount,
					<frame_system::Pallet<T>>::block_number() + T::MinStakingDuration::get(),
				))
				.map_err(|_| <Error<T>>::ExceedMaxUnstakings)?;

			Ok(())
		}

		fn unstake_deposit(ledger: &mut Ledger<T>, deposit: DepositId<T>) -> DispatchResult {
			ledger
				.unstaking_deposits
				.try_push((
					ledger.staked_deposits.remove(
						ledger
							.staked_deposits
							.iter()
							.position(|d| d == &deposit)
							.ok_or(<Error<T>>::DepositNotFound)?,
					),
					<frame_system::Pallet<T>>::block_number() + T::MinStakingDuration::get(),
				))
				.map_err(|_| <Error<T>>::ExceedMaxUnstakings)?;

			Ok(())
		}

		fn restake_ring(
			staked: &mut Balance,
			unstaking: &mut BoundedVec<(Balance, BlockNumberFor<T>), T::MaxUnstakings>,
			mut amount: Balance,
		) -> DispatchResult {
			let mut actual_restake = 0;

			// Cancel the latest `unstake` first.
			while let Some((u, _)) = unstaking.last_mut() {
				if let Some(k) = u.checked_sub(amount) {
					actual_restake += amount;
					*u = k;

					if k == 0 {
						unstaking
							.pop()
							.ok_or("[pallet::staking] record must exist, due to `last_mut`; qed")?;
					}

					break;
				} else {
					actual_restake += *u;
					amount -= *u;

					unstaking
						.pop()
						.ok_or("[pallet::staking] record must exist, due to `last_mut`; qed")?;
				}
			}

			*staked += actual_restake;

			Ok(())
		}

		fn restake_deposit(ledger: &mut Ledger<T>, deposit: DepositId<T>) -> DispatchResult {
			ledger
				.staked_deposits
				.try_push(
					ledger
						.unstaking_deposits
						.remove(
							ledger
								.unstaking_deposits
								.iter()
								.position(|(d, _)| d == &deposit)
								.ok_or(<Error<T>>::DepositNotFound)?,
						)
						.0,
				)
				.map_err(|_| <Error<T>>::ExceedMaxDeposits)?;

			Ok(())
		}

		fn claim_unstakings(who: &T::AccountId) -> DispatchResult {
			<Ledgers<T>>::try_mutate(who, |l| {
				let l = l.as_mut().ok_or(<Error<T>>::NotStaker)?;
				let now = <frame_system::Pallet<T>>::block_number();
				let mut r_claimed = 0;

				l.unstaking_ring.retain(|(a, t)| {
					if t <= &now {
						r_claimed += a;

						false
					} else {
						true
					}
				});
				<T as Config>::Ring::unstake(who, r_claimed)?;

				let mut d_claimed = Vec::new();

				l.unstaking_deposits.retain(|(d, t)| {
					if t <= &now {
						d_claimed.push(*d);

						false
					} else {
						true
					}
				});

				for d in d_claimed {
					T::Deposit::unstake(who, d)?;
				}

				Ok(())
			})
		}

		fn try_clean_ledger_of(who: &T::AccountId) {
			let _ = <Ledgers<T>>::try_mutate(who, |maybe_l| {
				let l = maybe_l.as_mut().ok_or(())?;

				if l.is_empty() {
					*maybe_l = None;

					<frame_system::Pallet<T>>::dec_consumers(who);

					Ok(())
				} else {
					Err(())
				}
			});
		}

		/// Update the record of block production.
		pub fn note_authors(authors: &[T::AccountId]) {
			<AuthoredBlocksCount<T>>::mutate(|(sum, map)| {
				authors.iter().cloned().for_each(|c| {
					*sum += One::one();

					map.entry(c).and_modify(|p_| *p_ += One::one()).or_insert(One::one());
				});
			});
		}

		/// Calculate the stakes of the given account.
		pub fn stake_of(who: &T::AccountId) -> Balance {
			<Ledgers<T>>::get(who)
				.map(|l| {
					l.staked_ring
						+ l.staked_deposits
							.into_iter()
							// We don't care if the deposit exists here.
							// It was guaranteed by the `stake`/`unstake`/`restake` functions.
							.fold(0, |r, d| r + T::Deposit::amount(who, d).unwrap_or_default())
				})
				.unwrap_or_default()
		}

		/// Distribute the session reward to staking pot and update the stakers' reward record.
		pub fn distribute_session_reward(amount: Balance) {
			let reward_to_ring = amount.saturating_div(2);
			let reward_to_kton = amount.saturating_sub(reward_to_ring);
			let kton_staker = T::KtonStaker::get();

			if T::IssuingManager::reward(&kton_staker, reward_to_kton).is_err() {
				Self::deposit_event(Event::Unpaid {
					who: kton_staker,
					amount: reward_to_kton,
				});
			}

			let (sum, map) = <AuthoredBlocksCount<T>>::take();
			let staking_pot = account_id();
			let mut actual_reward_to_ring = 0;
			let mut unpaid_reward_to_ring = 0;

			map.into_iter().for_each(|(c, p)| {
				let r = Perbill::from_rational(p, sum) * reward_to_ring;

				<PendingRewards<T>>::mutate(&c, |u| *u = u.map(|u| u + r).or(Some(r)));

				if T::IssuingManager::reward(&staking_pot, r).is_ok() {
					actual_reward_to_ring += amount;
				} else {
					unpaid_reward_to_ring += amount;
				}
			});

			if unpaid != 0 {
				Self::deposit_event(Event::Unpaid {
					who: staking_pot,
					amount: unpaid_reward_to_ring,
				});
			}
		}

		/// Pay the reward to the collator and its nominators.
		pub fn payout_inner(collator: T::AccountId) -> Result<Weight, DispatchError> {
			let c_exposure =
				call_on_exposure!(<Previous<T>>::take(&collator).ok_or(<Error<T>>::NoReward)?)?;
			let c_total_payout =
				<PendingRewards<T>>::take(&collator).ok_or(<Error<T>>::NoReward)?;
			let mut c_payout = c_exposure.commission * c_total_payout;
			let n_payout = c_total_payout - c_payout;
			for n_exposure in c_exposure.nominators {
				let n_payout = Perbill::from_rational(n_exposure.vote, c_exposure.vote) * n_payout;

				if collator == n_exposure.who {
					// If the collator nominated themselves.

					c_payout += n_payout;
				} else if T::IssuingManager::reward(&n_exposure.who, n_payout).is_ok() {
					Self::deposit_event(Event::Payout { who: n_exposure.who, amount: n_payout });
				} else {
					Self::deposit_event(Event::Unpaid { who: n_exposure.who, amount: n_payout });
				}
			}

			if T::IssuingManager::reward(&collator, c_payout).is_ok() {
				Self::deposit_event(Event::Payout { who: collator, amount: c_payout });
			} else {
				Self::deposit_event(Event::Unpaid { who: collator, amount: c_payout });
			}

			Ok(<T as Config>::WeightInfo::payout())
		}

		/// Prepare the session state.
		pub fn prepare_new_session(index: u32) -> Option<Vec<T::AccountId>> {
			<Pallet<T>>::shift_exposure_cache_states();

			#[allow(deprecated)]
			if call_on_exposure!(<Next<T>>::remove_all(None)).is_err() {
				return None;
			}

			let bn = <frame_system::Pallet<T>>::block_number();

			log::info!(
				"[pallet::staking] assembling new collators for new session {index} at #{bn:?}",
			);

			if let Ok(cs) = Self::elect() {
				if !cs.is_empty() {
					// TODO?: if we really need this event
					Self::deposit_event(Event::Elected { collators: cs.clone() });

					return Some(cs);
				}
			}

			// This error log is acceptable when testing with `genesis_collator = false`.
			log::error!(
				"[pallet::staking] fail to elect collators for new session {index} at #{bn:?}"
			);

			// Impossible case.
			//
			// But if there is an issue, retain the old collators; do not alter the session
			// collators if any error occurs to prevent the chain from stalling.
			None
		}

		/// Shift the exposure cache states.
		///
		/// Previous Current  Next
		/// Next     Previous Current
		/// Current  Next     Previous
		///
		/// ```nocompile
		/// loop { mutate(2, 0, 1) }
		/// ```
		pub fn shift_exposure_cache_states() {
			let (s0, s1, s2) = <ExposureCacheStates<T>>::get();

			<ExposureCacheStates<T>>::put((s2, s0, s1));
		}

		/// Elect the new collators.
		///
		/// This should only be called by the [`pallet_session::SessionManager::new_session`].
		pub fn elect() -> Result<Vec<T::AccountId>, DispatchError> {
			let nominators = <Nominators<T>>::iter().collect::<Vec<_>>();
			let mut collators = <Collators<T>>::iter()
				.map(|(c, cm)| {
					let scaler = Perbill::one() - cm;
					let mut collator_v = 0;
					let nominators = nominators
						.iter()
						.filter_map(|(n, c_)| {
							if c_ == &c {
								let nominator_v = scaler * Self::stake_of(n);

								collator_v += nominator_v;

								Some(IndividualExposure { who: n.to_owned(), vote: nominator_v })
							} else {
								None
							}
						})
						.collect();

					((c, Exposure { commission: cm, vote: collator_v, nominators }), collator_v)
				})
				.collect::<Vec<_>>();

			collators.sort_by(|(_, a), (_, b)| b.cmp(a));

			let cache_states = <ExposureCacheStates<T>>::get();

			collators
				.into_iter()
				.take(<CollatorCount<T>>::get() as _)
				.map(|((c, e), _)| {
					call_on_exposure!(cache_states, <Next<T>>::insert(&c, e))
						.map(|_| c)
						.map_err(Into::into)
				})
				.collect()
		}
	}
}
pub use pallet::*;

type DepositId<T> = <<T as Config>::Deposit as Stake>::Item;

/// Issuing and reward manager.
pub trait IssuingManager<T>
where
	T: Config,
{
	/// Generic session termination procedures.
	fn on_session_end() {
		let inflation = Self::inflate();
		let reward = Self::calculate_reward(inflation);

		<Pallet<T>>::distribute_session_reward(reward);
	}

	/// Inflation settings.
	fn inflate() -> Balance {
		0
	}

	/// Calculate the reward.
	fn calculate_reward(issued: Balance) -> Balance;

	/// The reward function.
	fn reward(who: &T::AccountId, amount: Balance) -> DispatchResult;
}
impl<T> IssuingManager<T> for ()
where
	T: Config,
{
	fn calculate_reward(_inflation: Balance) -> Balance {
		0
	}

	fn reward(_who: &T::AccountId, _amount: Balance) -> DispatchResult {
		Ok(())
	}
}

/// Exposure cache's state.
#[allow(missing_docs)]
#[cfg_attr(any(test, feature = "runtime-benchmarks", feature = "try-runtime"), derive(PartialEq))]
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
pub enum ExposureCacheState {
	Previous,
	Current,
	Next,
}

/// A convertor from collators id. Since this pallet does not have stash/controller, this is
/// just identity.
pub struct IdentityCollator;
impl<T> Convert<T, Option<T>> for IdentityCollator {
	fn convert(t: T) -> Option<T> {
		Some(t)
	}
}

/// Staking ledger.
#[derive(DebugNoBound, PartialEqNoBound, EqNoBound, Encode, Decode, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Ledger<T>
where
	T: Config,
{
	/// Staked RING.
	pub staked_ring: Balance,
	/// Staked deposits.
	pub staked_deposits: BoundedVec<DepositId<T>, <T as Config>::MaxDeposits>,
	/// The RING in unstaking process.
	pub unstaking_ring: BoundedVec<(Balance, BlockNumberFor<T>), T::MaxUnstakings>,
	/// The deposit in unstaking process.
	pub unstaking_deposits: BoundedVec<(DepositId<T>, BlockNumberFor<T>), T::MaxUnstakings>,
}
impl<T> Ledger<T>
where
	T: Config,
{
	fn is_empty(&self) -> bool {
		self.staked_ring == 0
			&& self.staked_deposits.is_empty()
			&& self.unstaking_ring.is_empty()
			&& self.unstaking_deposits.is_empty()
	}
}

/// A snapshot of the stake backing a single collator in the system.
#[cfg_attr(test, derive(Clone))]
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
pub struct Exposure<AccountId> {
	/// The commission of this collator.
	pub commission: Perbill,
	/// The total vote backing this collator.
	pub vote: Balance,
	/// Nominator staking map.
	pub nominators: Vec<IndividualExposure<AccountId>>,
}
/// A snapshot of the staker's state.
#[cfg_attr(test, derive(Clone))]
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
pub struct IndividualExposure<AccountId> {
	/// Nominator.
	pub who: AccountId,
	/// Nominator's staking vote.
	pub vote: Balance,
}

// Add reward points to block authors:
// - 20 points to the block producer for producing a (non-uncle) block in the parachain chain,
// - 2 points to the block producer for each reference to a previously unreferenced uncle, and
// - 1 point to the producer of each referenced uncle block.
impl<T> pallet_authorship::EventHandler<T::AccountId, BlockNumberFor<T>> for Pallet<T>
where
	T: Config + pallet_authorship::Config + pallet_session::Config,
{
	fn note_author(author: T::AccountId) {
		Self::note_authors(&[author])
	}
}

// Play the role of the session manager.
impl<T> pallet_session::SessionManager<T::AccountId> for Pallet<T>
where
	T: Config,
{
	fn end_session(_: u32) {
		T::IssuingManager::on_session_end();
	}

	fn start_session(_: u32) {}

	fn new_session(index: u32) -> Option<Vec<T::AccountId>> {
		Self::prepare_new_session(index)
	}
}

/// The account of the staking pot.
pub fn account_id<A>() -> A
where
	A: FullCodec,
{
	PalletId(*b"da/staki").into_account_truncating()
}
