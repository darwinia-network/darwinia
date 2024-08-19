// This file is part of Darwinia.
//
// Copyright (C) Darwinia Network
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

// TODO: check this after the migration is complete.
#![allow(clippy::needless_borrows_for_generic_args)]
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
#![allow(clippy::needless_borrows_for_generic_args)]

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
use ethabi::{Function, Param, ParamType, StateMutability, Token};
// darwinia
use dc_types::{Balance, Moment, UNIT};
// polkadot-sdk
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, UnixTime},
	DefaultNoBound, EqNoBound, PalletId, PartialEqNoBound,
};
use frame_system::{pallet_prelude::*, RawOrigin};
use pallet_session::ShouldEndSession as _;
use sp_core::{H160, U256};
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
			($crate::CacheState::$s, _, _) => Ok(<$crate::ExposureCache0<$t>>$($f)*),
			(_, $crate::CacheState::$s, _) => Ok(<$crate::ExposureCache1<$t>>$($f)*),
			(_, _, $crate::CacheState::$s) => Ok(<$crate::ExposureCache2<$t>>$($f)*),
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
/// Make it easier to call a function on a specific collators storage.
#[macro_export]
macro_rules! call_on_cache {
	($s_e:expr, <$s:ident<$t:ident>>$($f:tt)*) => {{
		match $s_e {
			($crate::CacheState::$s, _, _) => Ok(<$crate::CollatorsCache0<$t>>$($f)*),
			(_, $crate::CacheState::$s, _) => Ok(<$crate::CollatorsCache1<$t>>$($f)*),
			(_, _, $crate::CacheState::$s) => Ok(<$crate::CollatorsCache2<$t>>$($f)*),
			_ => {
				log::error!("[pallet::staking] collators cache states must be correct; qed");

				Err("[pallet::staking] collators cache states must be correct; qed")
			},
		}
	}};
	(<$s:ident<$t:ident>>$($f:tt)*) => {{
		let s = <$crate::CollatorsCacheState<$t>>::get();

		$crate::call_on_cache!(s, <$s<$t>>$($f)*)
	}};
}

type DepositId<T> = <<T as Config>::Deposit as Stake>::Item;

const PAYOUT_FRAC: Perbill = Perbill::from_percent(40);
const DAY_IN_MILLIS: Moment = 24 * 60 * 60 * 1_000;

#[frame_support::pallet]
pub mod pallet {
	// darwinia
	use crate::*;

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

		/// Unix time interface.
		type UnixTime: UnixTime;

		/// Pass [`pallet_session::Config::ShouldEndSession`]'s result to here.
		type ShouldEndSession: Get<bool>;

		/// Currency interface to pay the reward.
		type Currency: Currency<Self::AccountId, Balance = Balance>;

		/// Inflation and reward manager.
		type IssuingManager: IssuingManager<Self>;

		/// RING [`Stake`] interface.
		type Ring: Stake<AccountId = Self::AccountId, Item = Balance>;

		/// Deposit [`StakeExt`] interface.
		type Deposit: StakeExt<AccountId = Self::AccountId, Amount = Balance>;

		/// RING staking interface.
		type RingStaking: Election<Self::AccountId> + Reward<Self::AccountId>;

		/// KTON staking interface.
		type KtonStaking: Reward<Self::AccountId>;

		/// Treasury address.
		type Treasury: Get<Self::AccountId>;

		/// Maximum deposit count.
		#[pallet::constant]
		type MaxDeposits: Get<u32>;
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
		/// Exceed rate limit.
		ExceedRateLimit,
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
		(CacheState, CacheState, CacheState),
		ValueQuery,
		ExposureCacheStatesDefault<T>,
	>;
	/// Default value for [`ExposureCacheStates`].
	#[pallet::type_value]
	pub fn ExposureCacheStatesDefault<T: Config>() -> (CacheState, CacheState, CacheState) {
		(CacheState::Previous, CacheState::Current, CacheState::Next)
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

	/// Rate limit.
	///
	/// The maximum amount of RING that can be staked or unstaked in one session.
	#[pallet::storage]
	#[pallet::getter(fn rate_limit)]
	pub type RateLimit<T: Config> = StorageValue<_, Balance, ValueQuery>;

	/// Rate limit state.
	///
	/// Tracks the rate limit state in a session.
	#[pallet::storage]
	#[pallet::getter(fn rate_limit_state)]
	pub type RateLimitState<T: Config> = StorageValue<_, RateLimiter, ValueQuery>;

	/// RING staking contract address.
	#[pallet::storage]
	#[pallet::getter(fn ring_staking_contract)]
	pub type RingStakingContract<T: Config> = StorageValue<_, T::AccountId>;
	/// KTON staking contract address.
	#[pallet::storage]
	#[pallet::getter(fn kton_staking_contract)]
	pub type KtonStakingContract<T: Config> = StorageValue<_, T::AccountId>;

	/// Migration start point.
	#[pallet::storage]
	#[pallet::getter(fn migration_start_point)]
	pub type MigrationStartPoint<T: Config> = StorageValue<_, Moment, ValueQuery>;

	/// Collator cache state.
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
	#[pallet::getter(fn collator_cache_state)]
	pub type CollatorsCacheState<T: Config> = StorageValue<
		_,
		(CacheState, CacheState, CacheState),
		ValueQuery,
		CollatorCacheStateDefault<T>,
	>;
	/// Default value for [`CollatorsCacheState`].
	#[pallet::type_value]
	pub fn CollatorCacheStateDefault<T: Config>() -> (CacheState, CacheState, CacheState) {
		(CacheState::Previous, CacheState::Current, CacheState::Next)
	}

	// TODO: use `BoundedVec`.
	/// Exposure cache 0.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn collators_cache_0)]
	pub type CollatorsCache0<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	/// Exposure cache 1.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn collators_cache_1)]
	pub type CollatorsCache1<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	/// Exposure cache 2.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn collators_cache_2)]
	pub type CollatorsCache2<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[derive(DefaultNoBound)]
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// Current timestamp.
		pub now: Moment,
		/// The running time of Darwinia1.
		pub elapsed_time: Moment,
		/// Rate limit.
		pub rate_limit: Balance,
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
			<RateLimit<T>>::put(self.rate_limit);
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

			let flow_in_amount = <Ledgers<T>>::try_mutate(&who, |l| {
				let l = if let Some(l) = l {
					l
				} else {
					<frame_system::Pallet<T>>::inc_consumers(&who)?;

					*l = Some(Ledger { ring: Default::default(), deposits: Default::default() });

					l.as_mut().expect("[pallet::staking] `l` must be some; qed")
				};
				let mut v = ring_amount;

				if ring_amount != 0 {
					Self::stake_ring(&who, &mut l.ring, ring_amount)?;
				}

				for d in deposits.clone() {
					v = v.saturating_add(T::Deposit::amount(&who, d).unwrap_or_default());

					Self::stake_deposit(&who, l, d)?;
				}

				<Result<_, DispatchError>>::Ok(v)
			})?;

			if let Some(r) =
				<RateLimitState<T>>::get().flow_in(flow_in_amount, <RateLimit<T>>::get())
			{
				<RateLimitState<T>>::put(r);
			} else {
				Err(<Error<T>>::ExceedRateLimit)?;
			}

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

			let flow_out_amount = <Ledgers<T>>::try_mutate(&who, |l| {
				let l = l.as_mut().ok_or(<Error<T>>::NotStaker)?;
				let mut v = ring_amount;

				if ring_amount != 0 {
					l.ring = l
						.ring
						.checked_sub(ring_amount)
						.ok_or("[pallet::staking] `u128` must not be overflowed; qed")?;

					<T as Config>::Ring::unstake(&who, ring_amount)?;
				}

				for d in deposits {
					v = v.saturating_add(T::Deposit::amount(&who, d).unwrap_or_default());

					l.deposits.remove(
						l.deposits
							.iter()
							.position(|d_| d_ == &d)
							.ok_or(<Error<T>>::DepositNotFound)?,
					);

					T::Deposit::unstake(&who, d)?;
				}

				<Result<_, DispatchError>>::Ok(v)
			})?;

			if let Some(r) =
				<RateLimitState<T>>::get().flow_out(flow_out_amount, <RateLimit<T>>::get())
			{
				<RateLimitState<T>>::put(r);
			} else {
				Err(<Error<T>>::ExceedRateLimit)?;
			}

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

		/// Set the collator count.
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

		/// Set the max unstake RING limit.
		#[pallet::call_index(9)]
		#[pallet::weight(<T as Config>::WeightInfo::set_rate_limit())]
		pub fn set_rate_limit(origin: OriginFor<T>, amount: Balance) -> DispatchResult {
			ensure_root(origin)?;

			<RateLimit<T>>::put(amount);

			Ok(())
		}

		/// Set the RING reward distribution contract address.
		#[pallet::call_index(11)]
		#[pallet::weight(<T as Config>::WeightInfo::set_ring_staking_contract())]
		pub fn set_ring_staking_contract(
			origin: OriginFor<T>,
			address: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;

			<RingStakingContract<T>>::put(address);

			Ok(())
		}

		/// Set the KTON reward distribution contract address.
		#[pallet::call_index(10)]
		#[pallet::weight(<T as Config>::WeightInfo::set_kton_staking_contract())]
		pub fn set_kton_staking_contract(
			origin: OriginFor<T>,
			address: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;

			<KtonStakingContract<T>>::put(address);

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

			ledger.deposits.try_push(deposit).map_err(|_| <Error<T>>::ExceedMaxDeposits)?;

			Ok(())
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
			<AuthoredBlocksCount<T>>::mutate(|(total, map)| {
				authors.iter().cloned().for_each(|c| {
					*total += One::one();

					map.entry(c).and_modify(|p_| *p_ += One::one()).or_insert(One::one());
				});
			});
		}

		/// Calculate the stakes of the given account.
		pub fn stake_of(who: &T::AccountId) -> Balance {
			<Ledgers<T>>::get(who)
				.map(|l| {
					l.ring
						+ l.deposits
							.into_iter()
							// We don't care if the deposit exists here.
							// It was guaranteed by the `stake`/`unstake`/`restake` functions.
							.fold(0, |r, d| r + T::Deposit::amount(who, d).unwrap_or_default())
				})
				.unwrap_or_default()
		}

		/// Distribute the session reward to staking pot and update the stakers' reward record.
		pub fn distribute_session_reward(amount: Balance) {
			let who = <T as Config>::Treasury::get();

			if T::IssuingManager::reward(&who, amount).is_ok() {
				Self::deposit_event(Event::Payout { who, amount });
			} else {
				Self::deposit_event(Event::Unpaid { who, amount });
			}

			let reward_r = amount.saturating_div(2);
			let reward_k = amount.saturating_sub(reward_r);
			let (b_total, map) = <AuthoredBlocksCount<T>>::take();
			let collators_v2 = call_on_cache!(<Current<T>>::get()).unwrap_or_default();

			map.into_iter().for_each(|(c, b)| {
				let r = Perbill::from_rational(b, b_total).mul_floor(reward_r);

				if collators_v2.contains(&c) {
					T::RingStaking::distribute(Some(c), r);
				} else {
					<PendingRewards<T>>::mutate(c, |u| *u = u.map(|u| u + r).or(Some(r)));
				}
			});

			T::KtonStaking::distribute(None, reward_k);
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

				dbg!(n_payout);

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
			<RateLimitState<T>>::kill();
			<Pallet<T>>::shift_cache_states();

			#[allow(deprecated)]
			call_on_exposure!(<Next<T>>::remove_all(None)).ok()?;
			call_on_cache!(<Next<T>>::kill()).ok()?;

			let bn = <frame_system::Pallet<T>>::block_number();

			log::info!(
				"[pallet::staking] assembling new collators for new session {index} at #{bn:?}",
			);

			let (n1, n2) = Self::elect_ns();
			let cs_from_contract = Self::try_elect(n1, Self::elect_from_contract);
			let cs_from_pallet = Self::try_elect(n2, Self::elect);

			if n1 != cs_from_contract.len() as u32 || n2 != cs_from_pallet.len() as u32 {
				log::error!(
					"[pallet::staking] collator count mismatch; \
					expected collator count from contract: {n1}, from pallet: {n2}, \
					actual collator count from contract: {}, from pallet: {}",
					cs_from_contract.len(),
					cs_from_pallet.len(),
				);

				return None;
			}

			let cs = [cs_from_contract, cs_from_pallet].concat();

			if cs.is_empty() {
				// This error log is acceptable when testing with `genesis_collator = false`.
				log::error!(
					"[pallet::staking] fail to elect collators for new session {index} at #{bn:?}"
				);

				// Impossible case.
				//
				// But if there is an issue, retain the old collators; do not alter the session
				// collators if any error occurs to prevent the chain from stalling.
				None
			} else {
				// ? if we really need this event.
				Self::deposit_event(Event::Elected { collators: cs.clone() });

				Some(cs)
			}
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
		pub fn shift_cache_states() {
			let (s0, s1, s2) = <ExposureCacheStates<T>>::get();

			<ExposureCacheStates<T>>::put((s2, s0, s1));

			let (s0, s1, s2) = <CollatorsCacheState<T>>::get();

			<CollatorsCacheState<T>>::put((s2, s0, s1));
		}

		/// Elect the new collators.
		///
		/// This should only be called by the [`pallet_session::SessionManager::new_session`].
		pub fn elect(n: u32) -> Option<Vec<T::AccountId>> {
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
				.take(n as _)
				.map(|((c, e), _)| {
					call_on_exposure!(cache_states, <Next<T>>::insert(&c, e)).map(|_| c).ok()
				})
				.collect()
		}
	}
	// Implementation part.2.
	//
	// After the migration is completed,
	// the following implementation blocks will be merged into one.
	impl<T> Pallet<T>
	where
		T: Config,
	{
		/// Elect the new collators.
		pub fn elect_ns() -> (u32, u32) {
			let n = <CollatorCount<T>>::get();
			let n1 = Self::migration_progress() * n;

			(n1, n - n1)
		}

		fn migration_progress() -> Perbill {
			const TOTAL: Moment = 30 * 2 * DAY_IN_MILLIS;

			let start = <MigrationStartPoint<T>>::get();

			Perbill::from_rational(now::<T>() - start, TOTAL)
		}

		fn try_elect<F, R>(n: u32, elect: F) -> R
		where
			F: FnOnce(u32) -> Option<R>,
			R: Default,
		{
			if n > 0 {
				elect(n).unwrap_or_default()
			} else {
				Default::default()
			}
		}

		fn elect_from_contract(n: u32) -> Option<Vec<T::AccountId>> {
			let winners = T::RingStaking::elect(n)?;

			call_on_cache!(<Next<T>>::put(winners.clone())).ok()?;

			Some(winners)
		}
	}
	impl<T> pallet_authorship::EventHandler<T::AccountId, BlockNumberFor<T>> for Pallet<T>
	where
		T: Config + pallet_authorship::Config + pallet_session::Config,
	{
		fn note_author(author: T::AccountId) {
			Self::note_authors(&[author])
		}
	}
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
}
pub use pallet::*;

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
	fn calculate_reward(_: Balance) -> Balance {
		0
	}

	/// The reward function.
	fn reward(_: &T::AccountId, _: Balance) -> DispatchResult {
		Ok(())
	}
}
impl<T> IssuingManager<T> for () where T: Config {}

/// Election interface.
pub trait Election<AccountId> {
	/// Elect the new collators.
	fn elect(_: u32) -> Option<Vec<AccountId>> {
		None
	}
}
impl<AccountId> Election<AccountId> for () {}

/// Distribute the reward to a contract.
pub trait Reward<AccountId> {
	/// Distribute the reward.
	fn distribute(_: Option<AccountId>, _: Balance) {}
}
impl<AccountId> Reward<AccountId> for () {}

/// Staking rate limiter.
#[derive(Clone, Debug, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum RateLimiter {
	/// Positive balance.
	Pos(Balance),
	/// Negative balance.
	Neg(Balance),
}
impl RateLimiter {
	fn flow_in(self, amount: Balance, limit: Balance) -> Option<Self> {
		match self {
			Self::Pos(v) => v.checked_add(amount).filter(|&v| v <= limit).map(Self::Pos),
			Self::Neg(v) =>
				if v >= amount {
					Some(Self::Neg(v - amount))
				} else {
					let v = amount - v;

					if v <= limit {
						Some(Self::Pos(v))
					} else {
						None
					}
				},
		}
	}

	fn flow_out(self, amount: Balance, limit: Balance) -> Option<Self> {
		match self {
			Self::Pos(v) =>
				if v >= amount {
					Some(Self::Pos(v - amount))
				} else {
					let v = amount - v;

					if v <= limit {
						Some(Self::Neg(v))
					} else {
						None
					}
				},
			Self::Neg(v) => v.checked_add(amount).filter(|&new_v| new_v <= limit).map(Self::Neg),
		}
	}
}
impl Default for RateLimiter {
	fn default() -> Self {
		Self::Pos(0)
	}
}

/// Cache state.
#[allow(missing_docs)]
#[cfg_attr(any(test, feature = "runtime-benchmarks", feature = "try-runtime"), derive(PartialEq))]
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
pub enum CacheState {
	Previous,
	Current,
	Next,
}

/// Session ending checker.
pub struct ShouldEndSession<T>(PhantomData<T>);
impl<T> Get<bool> for ShouldEndSession<T>
where
	T: frame_system::Config + pallet_session::Config,
{
	fn get() -> bool {
		<T as pallet_session::Config>::ShouldEndSession::should_end_session(
			<frame_system::Pallet<T>>::block_number(),
		)
	}
}

/// Issue new token from pallet-balances.
pub struct BalanceIssuing<T>(PhantomData<T>);
impl<T> IssuingManager<T> for BalanceIssuing<T>
where
	T: Config,
{
	fn inflate() -> Balance {
		let now = now::<T>() as Moment;
		let session_duration = now - <SessionStartTime<T>>::get();
		let elapsed_time = <ElapsedTime<T>>::mutate(|t| {
			*t = t.saturating_add(session_duration);

			*t
		});

		<SessionStartTime<T>>::put(now);

		dc_inflation::issuing_in_period(session_duration, elapsed_time).unwrap_or_default()
	}

	fn calculate_reward(issued: Balance) -> Balance {
		PAYOUT_FRAC * issued
	}

	fn reward(who: &T::AccountId, amount: Balance) -> DispatchResult {
		let _ = T::Currency::deposit_creating(who, amount);

		Ok(())
	}
}

/// Transfer issued token from pallet-treasury.
pub struct TreasuryIssuing<T>(PhantomData<T>);
impl<T> IssuingManager<T> for TreasuryIssuing<T>
where
	T: Config,
{
	fn calculate_reward(_: Balance) -> Balance {
		20_000 * UNIT
	}

	fn reward(who: &T::AccountId, amount: Balance) -> DispatchResult {
		let treasury = <T as Config>::Treasury::get();

		if who == &treasury {
			Ok(())
		} else {
			T::Currency::transfer(
				&treasury,
				who,
				amount,
				frame_support::traits::ExistenceRequirement::KeepAlive,
			)
		}
	}
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
	pub ring: Balance,
	/// Staked deposits.
	pub deposits: BoundedVec<DepositId<T>, <T as Config>::MaxDeposits>,
}
impl<T> Ledger<T>
where
	T: Config,
{
	fn is_empty(&self) -> bool {
		self.ring == 0 && self.deposits.is_empty()
	}
}

/// A snapshot of the stake backing a single collator in the system.
#[cfg_attr(test, derive(Clone))]
#[derive(Encode, Decode, TypeInfo, RuntimeDebug)]
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
#[derive(Encode, Decode, TypeInfo, RuntimeDebug)]
pub struct IndividualExposure<AccountId> {
	/// Nominator.
	pub who: AccountId,
	/// Nominator's staking vote.
	pub vote: Balance,
}

/// RING staking interface.
pub struct RingStaking<T>(PhantomData<T>);
impl<T> Election<T::AccountId> for RingStaking<T>
where
	T: Config + darwinia_ethtx_forwarder::Config,
	T::AccountId: From<H160> + Into<H160>,
{
	fn elect(n: u32) -> Option<Vec<T::AccountId>> {
		const ZERO: [u8; 20] = [0; 20];

		let Some(rsc) = <RingStakingContract<T>>::get() else {
			log::error!("RING staking contract must be some; qed");

			return None;
		};
		let rsc = rsc.into();
		#[allow(deprecated)]
		let function = Function {
			name: "getTopCollators".to_owned(),
			inputs: vec![Param {
				name: "k".to_owned(),
				kind: ParamType::Uint(256),
				internal_type: None,
			}],
			outputs: vec![Param {
				name: "collators".to_owned(),
				kind: ParamType::Array(Box::new(ParamType::Address)),
				internal_type: None,
			}],
			constant: None,
			state_mutability: StateMutability::View,
		};
		let input = function
			.encode_input(&[Token::Int(n.into())])
			.map_err(|e| log::error!("failed to encode input due to {e:?}"))
			.ok()?;

		<darwinia_ethtx_forwarder::Pallet<T>>::forward_call(
			<T as Config>::Treasury::get().into(),
			rsc,
			input,
			Default::default(),
			U256::from(10_000_000_u64),
		)
		.map_err(|e| log::error!("failed to forward call due to {e:?}"))
		.ok()
		.and_then(|i| {
			function
				.decode_output(&i.value)
				.map_err(|e| log::error!("failed to decode output due to {e:?}"))
				.ok()
				.map(|tokens| {
					tokens
						.into_iter()
						.filter_map(|token| match token {
							Token::Address(addr) if addr.0 != ZERO =>
								Some(T::AccountId::from(addr)),
							_ => None,
						})
						.collect()
				})
		})
	}
}
// Distribute the reward to RING staking contract.
//
// https://github.com/darwinia-network/DIP-7/blob/7fa307136586f06c6911ce98d16c88689d91ba8c/src/collator/CollatorStakingHub.sol#L142.
impl<T> Reward<T::AccountId> for RingStaking<T>
where
	T: Config + darwinia_ethtx_forwarder::Config,
	T::AccountId: Into<H160>,
{
	fn distribute(who: Option<T::AccountId>, amount: Balance) {
		let Some(who) = who else {
			log::error!("who must be some; qed");

			return;
		};
		let Some(rsc) = <RingStakingContract<T>>::get() else {
			log::error!("RING staking contract must be some; qed");

			return;
		};
		let rsc = rsc.into();

		#[allow(deprecated)]
		darwinia_ethtx_forwarder::quick_forward_transact::<T>(
			<T as Config>::Treasury::get().into(),
			Function {
				name: "distributeReward".into(),
				inputs: vec![Param {
					name: "address".to_owned(),
					kind: ParamType::Address,
					internal_type: None,
				}],
				outputs: Vec::new(),
				constant: None,
				state_mutability: StateMutability::Payable,
			},
			&[Token::Address(who.into())],
			rsc,
			amount.into(),
			1_000_000.into(),
		);
	}
}

/// KTON staking interface.
pub struct KtonStaking<T>(PhantomData<T>);
// Distribute the reward to KTON staking contract.
//
// https://github.com/darwinia-network/KtonDAO/blob/2de20674f2ef90b749ade746d0768c7bda356402/src/staking/KtonDAOVault.sol#L40.
impl<T> Reward<T::AccountId> for KtonStaking<T>
where
	T: Config + darwinia_ethtx_forwarder::Config,
	T::AccountId: Into<H160>,
{
	fn distribute(_: Option<T::AccountId>, amount: Balance) {
		let Some(ksc) = <KtonStakingContract<T>>::get() else {
			log::error!("KTON staking contract must be some; qed");

			return;
		};
		let ksc = ksc.into();

		#[allow(deprecated)]
		darwinia_ethtx_forwarder::quick_forward_transact::<T>(
			<T as Config>::Treasury::get().into(),
			Function {
				name: "distributeRewards".into(),
				inputs: Vec::new(),
				outputs: vec![Param {
					name: "success or not".into(),
					kind: ParamType::Bool,
					internal_type: None,
				}],
				constant: None,
				state_mutability: StateMutability::Payable,
			},
			&[],
			ksc,
			amount.into(),
			1_000_000.into(),
		);
	}
}

/// The account of the staking pot.
pub fn account_id<A>() -> A
where
	A: FullCodec,
{
	PalletId(*b"da/staki").into_account_truncating()
}

fn now<T>() -> Moment
where
	T: Config,
{
	T::UnixTime::now().as_millis()
}
