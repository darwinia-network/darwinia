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
//! - KTON: Darwinia's commitment token
//! - Deposit: Locking RINGs' ticket

// TODO: nomination upper limit

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
#![deny(unused_crate_dependencies)]

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
use frame_support::{pallet_prelude::*, DefaultNoBound, EqNoBound, PalletId, PartialEqNoBound};
use frame_system::{pallet_prelude::*, RawOrigin};
use sp_runtime::{
	traits::{AccountIdConversion, Convert},
	Perbill, Perquintill,
};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

#[frame_support::pallet]
pub mod pallet {
	// darwinia
	use crate::*;

	// Deposit helper for runtime benchmark.
	#[cfg(feature = "runtime-benchmarks")]
	use darwinia_deposit::Config as DepositConfig;
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

		/// KTON [`Stake`] interface.
		type Kton: Stake<AccountId = Self::AccountId, Item = Balance>;

		/// Deposit [`StakeExt`] interface.
		type Deposit: StakeExt<AccountId = Self::AccountId, Amount = Balance>;

		/// On session end handler.
		type OnSessionEnd: OnSessionEnd<Self>;

		/// Minimum time to stake at least.
		#[pallet::constant]
		type MinStakingDuration: Get<BlockNumberFor<Self>>;

		/// Maximum deposit count.
		#[pallet::constant]
		type MaxDeposits: Get<u32>;

		/// Maximum unstaking/unbonding count.
		#[pallet::constant]
		type MaxUnstakings: Get<u32>;
	}

	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An account has staked.
		Staked {
			staker: T::AccountId,
			ring_amount: Balance,
			kton_amount: Balance,
			deposits: Vec<DepositId<T>>,
		},
		/// An account has unstaked.
		Unstaked {
			staker: T::AccountId,
			ring_amount: Balance,
			kton_amount: Balance,
			deposits: Vec<DepositId<T>>,
		},
		CommissionUpdated {
			who: T::AccountId,
			commission: Perbill,
		},
		/// A payout has been made for the staker.
		Payout {
			staker: T::AccountId,
			amount: Balance,
		},
		/// Unable to pay the staker's reward.
		Unpaied {
			staker: T::AccountId,
			amount: Balance,
		},
		/// A new collator set has been elected.
		Elected {
			collators: Vec<T::AccountId>,
		},
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
	}

	/// All staking ledgers.
	#[pallet::storage]
	#[pallet::getter(fn ledger_of)]
	pub type Ledgers<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Ledger<T>>;

	/// Total staked RING.
	///
	/// This will count RING + deposit(locking RING).
	#[pallet::storage]
	#[pallet::getter(fn ring_pool)]
	pub type RingPool<T: Config> = StorageValue<_, Balance, ValueQuery>;

	/// Total staked KTON.
	#[pallet::storage]
	#[pallet::getter(fn kton_pool)]
	pub type KtonPool<T: Config> = StorageValue<_, Balance, ValueQuery>;

	/// The map from (wannabe) collator to the preferences of that collator.
	#[pallet::storage]
	#[pallet::getter(fn collator_of)]
	pub type Collators<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Perbill>;

	/// Current stakers' exposure.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn exposure_of)]
	pub type Exposures<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Exposure<T::AccountId>>;

	/// Next stakers' exposure.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn next_exposure_of)]
	pub type NextExposures<T: Config> =
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

	/// Collator's reward points.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn reward_points)]
	pub type RewardPoints<T: Config> =
		StorageValue<_, (RewardPoint, BTreeMap<T::AccountId, RewardPoint>), ValueQuery>;

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

			self.collators.iter().for_each(|(who, stake)| {
				<Pallet<T>>::stake(RawOrigin::Signed(who.to_owned()).into(), *stake, 0, Vec::new())
					.expect("[pallet::staking] 0, genesis must be built; qed");
				<Pallet<T>>::collect(RawOrigin::Signed(who.to_owned()).into(), Default::default())
					.expect("[pallet::staking] 1, genesis must be built; qed");
				<Pallet<T>>::nominate(RawOrigin::Signed(who.to_owned()).into(), who.to_owned())
					.expect("[pallet::staking] 2, genesis must be built; qed");
			});
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);
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
			kton_amount: Balance,
			deposits: Vec<DepositId<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			if ring_amount == 0 && kton_amount == 0 && deposits.is_empty() {
				return Ok(());
			}

			<Ledgers<T>>::try_mutate(&who, |l| {
				let l = if let Some(l) = l {
					l
				} else {
					<frame_system::Pallet<T>>::inc_consumers(&who)?;

					*l = Some(Ledger {
						staked_ring: Default::default(),
						staked_kton: Default::default(),
						staked_deposits: Default::default(),
						unstaking_ring: Default::default(),
						unstaking_kton: Default::default(),
						unstaking_deposits: Default::default(),
					});

					l.as_mut().expect("[pallet::staking] `l` must be some; qed")
				};

				if ring_amount != 0 {
					Self::stake_token::<<T as Config>::Ring, RingPool<T>>(
						&who,
						&mut l.staked_ring,
						ring_amount,
					)?;
				}
				if kton_amount != 0 {
					Self::stake_token::<<T as Config>::Kton, KtonPool<T>>(
						&who,
						&mut l.staked_kton,
						kton_amount,
					)?;
				}

				for d in deposits.clone() {
					Self::stake_deposit(&who, l, d)?;
				}

				DispatchResult::Ok(())
			})?;

			Self::deposit_event(Event::Staked { staker: who, ring_amount, kton_amount, deposits });

			Ok(())
		}

		/// Withdraw stakes from the staking pool.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::unstake(deposits.len() as _))]
		pub fn unstake(
			origin: OriginFor<T>,
			ring_amount: Balance,
			kton_amount: Balance,
			deposits: Vec<DepositId<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			if ring_amount == 0 && kton_amount == 0 && deposits.is_empty() {
				return Ok(());
			}

			<Ledgers<T>>::try_mutate(&who, |l| {
				let l = l.as_mut().ok_or(<Error<T>>::NotStaker)?;

				if ring_amount != 0 {
					Self::unstake_token::<RingPool<T>>(
						&mut l.staked_ring,
						&mut l.unstaking_ring,
						ring_amount,
					)?;
				}
				if kton_amount != 0 {
					Self::unstake_token::<KtonPool<T>>(
						&mut l.staked_kton,
						&mut l.unstaking_kton,
						kton_amount,
					)?;
				}

				for d in deposits {
					Self::unstake_deposit(&who, l, d)?;
				}

				DispatchResult::Ok(())
			})?;

			// TODO: event?

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
			kton_amount: Balance,
			deposits: Vec<DepositId<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			if ring_amount == 0 && kton_amount == 0 && deposits.is_empty() {
				return Ok(());
			}

			<Ledgers<T>>::try_mutate(&who, |l| {
				let l = l.as_mut().ok_or(<Error<T>>::NotStaker)?;

				if ring_amount != 0 {
					Self::restake_token::<RingPool<T>>(
						&mut l.staked_ring,
						&mut l.unstaking_ring,
						ring_amount,
					)?;
				}
				if kton_amount != 0 {
					Self::restake_token::<KtonPool<T>>(
						&mut l.staked_kton,
						&mut l.unstaking_kton,
						kton_amount,
					)?;
				}

				for d in deposits {
					Self::restake_deposit(&who, l, d)?;
				}

				DispatchResult::Ok(())
			})?;

			// TODO: event?

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

			// TODO: event?

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

			// TODO: event?

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

			// TODO: event?

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
		fn update_pool<P>(increase: bool, amount: Balance) -> DispatchResult
		where
			P: frame_support::StorageValue<Balance, Query = Balance>,
		{
			P::try_mutate(|p| {
				*p = if increase {
					p.checked_add(amount)
						.ok_or("[pallet::staking] `u128` must not be overflowed; qed")?
				} else {
					p.checked_sub(amount)
						.ok_or("[pallet::staking] `u128` must not be overflowed; qed")?
				};

				Ok(())
			})
		}

		fn stake_token<S, P>(
			who: &T::AccountId,
			record: &mut Balance,
			amount: Balance,
		) -> DispatchResult
		where
			S: Stake<AccountId = T::AccountId, Item = Balance>,
			P: frame_support::StorageValue<Balance, Query = Balance>,
		{
			S::stake(who, amount)?;

			*record = record
				.checked_add(amount)
				.ok_or("[pallet::staking] `u128` must not be overflowed; qed")?;

			Self::update_pool::<P>(true, amount)?;

			Ok(())
		}

		fn stake_deposit(
			who: &T::AccountId,
			ledger: &mut Ledger<T>,
			deposit: DepositId<T>,
		) -> DispatchResult {
			T::Deposit::stake(who, deposit)?;

			ledger.staked_deposits.try_push(deposit).map_err(|_| <Error<T>>::ExceedMaxDeposits)?;

			Self::update_pool::<RingPool<T>>(true, T::Deposit::amount(who, deposit)?)?;

			Ok(())
		}

		fn unstake_token<P>(
			staked: &mut Balance,
			unstaking: &mut BoundedVec<(Balance, BlockNumberFor<T>), T::MaxUnstakings>,
			amount: Balance,
		) -> DispatchResult
		where
			P: frame_support::StorageValue<Balance, Query = Balance>,
		{
			*staked = staked
				.checked_sub(amount)
				.ok_or("[pallet::staking] `u128` must not be overflowed; qed")?;

			unstaking
				.try_push((
					amount,
					<frame_system::Pallet<T>>::block_number() + T::MinStakingDuration::get(),
				))
				.map_err(|_| <Error<T>>::ExceedMaxUnstakings)?;

			Self::update_pool::<P>(false, amount)?;

			Ok(())
		}

		fn unstake_deposit(
			who: &T::AccountId,
			ledger: &mut Ledger<T>,
			deposit: DepositId<T>,
		) -> DispatchResult {
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

			Self::update_pool::<RingPool<T>>(false, T::Deposit::amount(who, deposit)?)?;

			Ok(())
		}

		fn restake_token<P>(
			staked: &mut Balance,
			unstaking: &mut BoundedVec<(Balance, BlockNumberFor<T>), T::MaxUnstakings>,
			mut amount: Balance,
		) -> DispatchResult
		where
			P: frame_support::StorageValue<Balance, Query = Balance>,
		{
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

			Self::update_pool::<P>(true, actual_restake)?;

			Ok(())
		}

		fn restake_deposit(
			who: &T::AccountId,
			ledger: &mut Ledger<T>,
			deposit: DepositId<T>,
		) -> DispatchResult {
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

			Self::update_pool::<RingPool<T>>(true, T::Deposit::amount(who, deposit)?)?;

			Ok(())
		}

		fn claim_unstakings(who: &T::AccountId) -> DispatchResult {
			<Ledgers<T>>::try_mutate(who, |l| {
				let l = l.as_mut().ok_or(<Error<T>>::NotStaker)?;
				let now = <frame_system::Pallet<T>>::block_number();
				let claim = |u: &mut BoundedVec<_, _>, c: &mut Balance| {
					u.retain(|(a, t)| {
						if t <= &now {
							*c += a;

							false
						} else {
							true
						}
					});
				};
				let mut r_claimed = 0;

				claim(&mut l.unstaking_ring, &mut r_claimed);
				<T as Config>::Ring::unstake(who, r_claimed)?;

				let mut k_claimed = 0;

				claim(&mut l.unstaking_kton, &mut k_claimed);
				<T as Config>::Kton::unstake(who, k_claimed)?;

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

		/// Add reward points to collators using their account id.
		pub fn reward_by_ids(collators: &[(T::AccountId, RewardPoint)]) {
			<RewardPoints<T>>::mutate(|(total, reward_map)| {
				collators.iter().cloned().for_each(|(c, p)| {
					*total += p;

					reward_map.entry(c).and_modify(|p_| *p_ += p).or_insert(p);
				});
			});
		}

		// Power is a mixture of RING and KTON.
		// - `total_ring_power = (amount / total_staked_ring) * HALF_POWER`
		// - `total_kton_power = (amount / total_staked_kton) * HALF_POWER`
		fn balance2power<P>(amount: Balance) -> Power
		where
			P: frame_support::StorageValue<Balance, Query = Balance>,
		{
			(Perquintill::from_rational(amount, P::get().max(1)) * 500_000_000_u128) as _
		}

		/// Calculate the power of the given account.
		pub fn power_of(who: &T::AccountId) -> Power {
			<Ledgers<T>>::get(who)
				.map(|l| {
					Self::balance2power::<RingPool<T>>(
						l.staked_ring
							+ l.staked_deposits
								.into_iter()
								// We don't care if the deposit exists here.
								// It was guaranteed by the `stake`/`unstake`/`restake` functions.
								.fold(0, |r, d| r + T::Deposit::amount(who, d).unwrap_or_default()),
					) + Self::balance2power::<KtonPool<T>>(l.staked_kton)
				})
				.unwrap_or_default()
		}

		// TODO: weight
		/// Pay the session reward to the stakers.
		pub fn payout(amount: Balance) -> Balance {
			let (total_points, reward_map) = <RewardPoints<T>>::get();
			// Due to the `payout * percent` there might be some losses.
			let mut actual_payout = 0;

			for (c, p) in reward_map {
				let Some(commission) = <Collators<T>>::get(&c) else {
						#[cfg(test)]
						panic!("[pallet::staking] collator({c:?}) must be found; qed");
						#[cfg(not(test))]
						{
							log::error!("[pallet::staking] collator({c:?}) must be found; qed");

							continue;
						}
					};
				let c_total_payout = Perbill::from_rational(p, total_points) * amount;
				let mut c_payout = commission * c_total_payout;
				let n_payout = c_total_payout - c_payout;
				let Some(c_exposure) = <Exposures<T>>::get(&c) else {
						#[cfg(test)]
						panic!("[pallet::staking] exposure({c:?}) must be found; qed");
						#[cfg(not(test))]
						{
							log::error!("[pallet::staking] exposure({c:?}) must be found; qed");

							continue;
						}
					};

				for n_exposure in c_exposure.nominators {
					let n_payout =
						Perbill::from_rational(n_exposure.vote, c_exposure.vote) * n_payout;

					if c == n_exposure.who {
						// If the collator nominated themselves.

						c_payout += n_payout;
					} else if T::OnSessionEnd::reward(&n_exposure.who, n_payout).is_ok() {
						actual_payout += n_payout;

						Self::deposit_event(Event::Payout {
							staker: n_exposure.who,
							amount: n_payout,
						});
					} else {
						Self::deposit_event(Event::Unpaied {
							staker: n_exposure.who,
							amount: n_payout,
						});
					}
				}

				if T::OnSessionEnd::reward(&c, c_payout).is_ok() {
					actual_payout += c_payout;

					Self::deposit_event(Event::Payout { staker: c, amount: c_payout });
				} else {
					Self::deposit_event(Event::Unpaied { staker: c, amount: c_payout });
				}
			}

			actual_payout
		}

		/// Prepare the session state.
		pub fn prepare_new_session() {
			<RewardPoints<T>>::kill();
			#[allow(deprecated)]
			<Exposures<T>>::remove_all(None);
			<NextExposures<T>>::iter().drain().for_each(|(k, v)| {
				<Exposures<T>>::insert(k, v);
			});
		}

		/// Elect the new collators.
		///
		/// This should only be called by the [`pallet_session::SessionManager::new_session`].
		pub fn elect() -> Vec<T::AccountId> {
			let mut collators = <Collators<T>>::iter()
				.map(|(c, cm)| {
					let scaler = Perbill::one() - cm;
					let mut collator_v = 0;
					let nominators = <Nominators<T>>::iter()
						.filter_map(|(n, c_)| {
							if c_ == c {
								let nominator_v = scaler * Self::power_of(&n);

								collator_v += nominator_v;

								Some(IndividualExposure { who: n, vote: nominator_v })
							} else {
								None
							}
						})
						.collect();

					((c, Exposure { vote: collator_v, nominators }), collator_v)
				})
				.collect::<Vec<_>>();

			collators.sort_by(|(_, a), (_, b)| b.cmp(a));

			collators
				.into_iter()
				.take(<CollatorCount<T>>::get() as _)
				.map(|((c, e), _)| {
					<NextExposures<T>>::insert(&c, e);

					c
				})
				.collect()
		}
	}
}
pub use pallet::*;

type RewardPoint = u32;
type Power = u32;
type Vote = u32;

type DepositId<T> = <<T as Config>::Deposit as Stake>::Item;

/// On session end handler.
///
/// Currently it is only used to control the inflation.
pub trait OnSessionEnd<T>
where
	T: Config,
{
	/// Generic session termination procedures.
	fn on_session_end() {
		let inflation = Self::inflate();
		let reward = Self::calculate_reward(inflation);
		let actual_payout = <Pallet<T>>::payout(reward);

		Self::clean(inflation.unwrap_or(reward).saturating_sub(actual_payout));
	}

	/// Inflation settings.
	fn inflate() -> Option<Balance> {
		None
	}

	/// Calculate the reward.
	fn calculate_reward(maybe_inflation: Option<Balance>) -> Balance;

	/// The reward function.
	fn reward(who: &T::AccountId, amount: Balance) -> DispatchResult;

	/// Clean the data; currently, only the unissued reward is present.
	fn clean(_unissued: Balance) {}
}
impl<T> OnSessionEnd<T> for ()
where
	T: Config,
{
	fn inflate() -> Option<Balance> {
		None
	}

	fn calculate_reward(_maybe_inflation: Option<Balance>) -> Balance {
		0
	}

	fn reward(_who: &T::AccountId, _amount: Balance) -> DispatchResult {
		Ok(())
	}

	fn clean(_unissued: Balance) {}
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
	/// Staked KTON.
	pub staked_kton: Balance,
	/// Staked deposits.
	pub staked_deposits: BoundedVec<DepositId<T>, <T as Config>::MaxDeposits>,
	/// The RING in unstaking process.
	pub unstaking_ring: BoundedVec<(Balance, BlockNumberFor<T>), T::MaxUnstakings>,
	/// The KTON in unstaking process.
	pub unstaking_kton: BoundedVec<(Balance, BlockNumberFor<T>), T::MaxUnstakings>,
	/// The deposit in unstaking process.
	pub unstaking_deposits: BoundedVec<(DepositId<T>, BlockNumberFor<T>), T::MaxUnstakings>,
}
impl<T> Ledger<T>
where
	T: Config,
{
	fn is_empty(&self) -> bool {
		self.staked_ring == 0
			&& self.staked_kton == 0
			&& self.staked_deposits.is_empty()
			&& self.unstaking_ring.is_empty()
			&& self.unstaking_kton.is_empty()
			&& self.unstaking_deposits.is_empty()
	}
}

// TODO: remove these
/// A snapshot of the stake backing a single collator in the system.
#[cfg(feature = "try-runtime")]
#[derive(PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
pub struct Exposure<AccountId>
where
	AccountId: PartialEq,
{
	/// The total vote backing this collator.
	pub vote: Vote,
	/// Nominator staking map.
	pub nominators: Vec<IndividualExposure<AccountId>>,
}
/// A snapshot of the stake backing a single collator in the system.
#[cfg(not(feature = "try-runtime"))]
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
pub struct Exposure<AccountId> {
	/// The total vote backing this collator.
	pub vote: Vote,
	/// Nominator staking map.
	pub nominators: Vec<IndividualExposure<AccountId>>,
}
/// A snapshot of the staker's state.
#[cfg(feature = "try-runtime")]
#[derive(PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
pub struct IndividualExposure<AccountId>
where
	AccountId: PartialEq,
{
	/// Nominator.
	pub who: AccountId,
	/// Nominator's staking vote.
	pub vote: Vote,
}
/// A snapshot of the staker's state.
#[cfg(not(feature = "try-runtime"))]
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
pub struct IndividualExposure<AccountId> {
	/// Nominator.
	pub who: AccountId,
	/// Nominator's staking vote.
	pub vote: Vote,
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
		Self::reward_by_ids(&[(author, 20)])
	}
}

// Play the role of the session manager.
impl<T> pallet_session::SessionManager<T::AccountId> for Pallet<T>
where
	T: Config,
{
	fn end_session(_: u32) {
		T::OnSessionEnd::on_session_end();
	}

	fn start_session(_: u32) {}

	fn new_session(index: u32) -> Option<Vec<T::AccountId>> {
		Self::prepare_new_session();

		log::info!(
			"[pallet::staking] assembling new collators for new session {} at #{:?}",
			index,
			<frame_system::Pallet<T>>::block_number(),
		);

		let collators = Self::elect();

		Self::deposit_event(Event::Elected { collators: collators.clone() });

		Some(collators)
	}
}

/// The account of the staking pot.
pub fn account_id<A>() -> A
where
	A: FullCodec,
{
	PalletId(*b"da/staki").into_account_truncating()
}
