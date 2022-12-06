// This file is part of Darwinia.
//
// Copyright (C) 2018-2022 Darwinia Network
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
//! This is a completely specialized stake pallet designed only for Darwinia parachain.
//! So, this pallet will eliminate the generic parameters as much as possible.
//!
//! ## Overview
//!
//! ### Acceptable stakes:
//! - RING: Darwinia's native token
//! - KTON: Darwinia's commitment token
//! - Deposit: Locking RINGs' ticket

// TODO: weight
// TODO: nomination upper limit

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

mod weights;
pub use weights::WeightInfo;

// core
use core::fmt::Debug;
// crates.io
use codec::FullCodec;
// darwinia
use dc_inflation::TOTAL_SUPPLY;
use dc_types::{Balance, Moment};
// substrate
use frame_support::{
	log,
	pallet_prelude::*,
	traits::{Currency, OnUnbalanced, UnixTime},
	EqNoBound, PalletId, PartialEqNoBound,
};
use frame_system::{pallet_prelude::*, RawOrigin};
use sp_runtime::{
	traits::{AccountIdConversion, Convert},
	Perbill, Perquintill,
};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

type RewardPoint = u32;
type Power = u32;

type DepositId<T> = <<T as Config>::Deposit as Stake>::Item;
type NegativeImbalance<T> = <<T as Config>::RingCurrency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

/// Stake trait that stake items must be implemented.
pub trait Stake {
	/// Account type.
	type AccountId;
	/// Stake item type.
	///
	/// Basically, it's just a num type.
	type Item: Clone + Copy + Debug + PartialEq + FullCodec + MaxEncodedLen + TypeInfo;

	/// Add stakes to the staking pool.
	///
	/// This will transfer the stakes to a pallet/contact account.
	fn stake(who: &Self::AccountId, item: Self::Item) -> DispatchResult;

	/// Withdraw stakes from the staking pool.
	///
	/// This will transfer the stakes back to the staker's account.
	fn unstake(who: &Self::AccountId, item: Self::Item) -> DispatchResult;
}
/// Extended stake trait.
///
/// Provide a way to access the deposit RING amount.
pub trait StakeExt: Stake {
	/// Amount type.
	type Amount;

	/// Get the staked amount.
	fn amount(who: &Self::AccountId, item: Self::Item) -> Self::Amount;
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
#[derive(PartialEqNoBound, EqNoBound, Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
#[scale_info(skip_type_params(T))]
pub struct Ledger<T>
where
	T: Config,
{
	/// Staker.
	pub account: T::AccountId,
	/// Staked RING.
	pub staked_ring: Balance,
	/// Staked KTON.
	pub staked_kton: Balance,
	/// Staked deposits.
	pub staked_deposits: BoundedVec<DepositId<T>, T::MaxDeposits>,
	/// The RING in unstaking process.
	pub unstaking_ring: BoundedVec<(Balance, T::BlockNumber), T::MaxUnstakings>,
	/// The KTON in unstaking process.
	pub unstaking_kton: BoundedVec<(Balance, T::BlockNumber), T::MaxUnstakings>,
	/// The deposit in unstaking process.
	pub unstaking_deposits: BoundedVec<(DepositId<T>, T::BlockNumber), T::MaxUnstakings>,
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

/// A snapshot of the stake backing a single collator in the system.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
pub struct Exposure<AccountId> {
	/// The total power backing this collator.
	pub total: Power,
	/// Collator's self stake power.
	pub own: Power,
	/// Nominators' stake power.
	pub others: Vec<IndividualExposure<AccountId>>,
}
/// A snapshot of the staker's state.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
pub struct IndividualExposure<AccountId> {
	/// Nominator.
	pub who: AccountId,
	/// Nominator's stake power.
	pub value: Power,
}

#[frame_support::pallet]
pub mod pallet {
	// darwinia
	use crate::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Override the [`frame_system::Config::RuntimeEvent`].
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Unix time getter.
		type UnixTime: UnixTime;

		/// RING [`Currency`] interface.
		///
		/// Only use for inflation.
		type RingCurrency: Currency<Self::AccountId, Balance = Balance>;

		/// Tokens have been minted and are unused for stakers reward.
		///
		/// Usually, it's treasury.
		type RewardRemainder: OnUnbalanced<NegativeImbalance<Self>>;

		/// RING [`Stake`] interface.
		type Ring: Stake<AccountId = Self::AccountId, Item = Balance>;

		/// KTON [`Stake`] interface.
		type Kton: Stake<AccountId = Self::AccountId, Item = Balance>;

		/// Deposit [`StakeExt`] interface.
		type Deposit: StakeExt<AccountId = Self::AccountId, Amount = Balance>;

		/// Minimum time to stake at least.
		#[pallet::constant]
		type MinStakingDuration: Get<Self::BlockNumber>;

		/// The percentage of the total payout that is distributed to stakers.
		///
		/// Usually, the rest goes to the treasury.
		#[pallet::constant]
		type PayoutFraction: Get<Perbill>;

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
		/// A payout has been made for the staker.
		Payout { staker: T::AccountId, ring_amount: Balance },
		/// A new collator set has been elected.
		Elected { collators: Vec<T::AccountId> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Exceed maximum deposit count.
		ExceedMaxDeposits,
		/// Exceed maximum unstaking/unbonding count.
		ExceedMaxUnstakings,
		/// You are not a staker.
		NotStaker,
		/// Target is not a collator.
		TargetNotCollator,
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

	/// Stakers' exposure.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn exposure)]
	pub type Exposures<T: Config> =
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
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { now: 0, elapsed_time: 0, collator_count: 0, collators: Vec::new() }
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<SessionStartTime<T>>::put(self.now);
			<ElapsedTime<T>>::put(self.elapsed_time);
			<CollatorCount<T>>::put(self.collator_count);

			self.collators.iter().cloned().for_each(|(who, stake)| {
				<Pallet<T>>::stake(RawOrigin::Signed(who.clone()).into(), stake, 0, Vec::new())
					.unwrap();
				<Pallet<T>>::collect(RawOrigin::Signed(who).into(), Default::default()).unwrap();
			});
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add stakes to the staking pool.
		///
		/// This will transfer the stakes to a pallet/contact account.
		#[pallet::weight(0)]
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
						account: who.to_owned(),
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
					Self::stake_ring(l, ring_amount)?;
				}
				if kton_amount != 0 {
					Self::stake_kton(l, kton_amount)?;
				}

				for d in deposits.clone() {
					Self::stake_deposit(l, d)?;
				}

				DispatchResult::Ok(())
			})?;

			Self::deposit_event(Event::<T>::Staked {
				staker: who,
				ring_amount,
				kton_amount,
				deposits,
			});

			Ok(())
		}

		/// Withdraw stakes from the staking pool.
		#[pallet::weight(0)]
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
				let Some(l) = l else {
					return Err(<Error<T>>::NotStaker)?;
				};

				if ring_amount != 0 {
					Self::unstake_ring(l, ring_amount)?;
				}
				if kton_amount != 0 {
					Self::unstake_kton(l, kton_amount)?;
				}

				for d in deposits {
					Self::unstake_deposit(l, d)?;
				}

				DispatchResult::Ok(())
			})?;

			// TODO: event?

			Ok(())
		}

		/// Claim the stakes from the pallet/contract account.
		#[pallet::weight(0)]
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
		#[pallet::weight(0)]
		pub fn collect(origin: OriginFor<T>, commission: Perbill) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::ensure_staker(&who)?;
			<Collators<T>>::mutate(&who, |c| *c = Some(commission));

			// TODO: event?

			Ok(())
		}

		/// Declare the desire to nominate a collator.
		///
		/// Effects will be felt at the beginning of the next session.
		#[pallet::weight(0)]
		pub fn nominate(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::ensure_staker(&who)?;

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
		#[pallet::weight(0)]
		pub fn chill(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			<Collators<T>>::remove(&who);
			<Nominators<T>>::remove(&who);

			// TODO: event?

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
				let p_new = if increase {
					let Some(p_new) = p.checked_add(amount) else {
						return Err("[pallet::staking] `u128` must not be overflowed; qed")?;
					};

					p_new
				} else {
					let Some(p_new) = p.checked_sub(amount) else {
						return Err("[pallet::staking] `u128` must not be overflowed; qed")?;
					};

					p_new
				};

				*p = p_new;

				Ok(())
			})
		}

		fn stake_ring(ledger: &mut Ledger<T>, amount: Balance) -> DispatchResult {
			T::Ring::stake(&ledger.account, amount)?;

			ledger.staked_ring = if let Some(r) = ledger.staked_ring.checked_add(amount) {
				r
			} else {
				return Err("[pallet::staking] `u128` must not be overflowed; qed")?;
			};

			Self::update_pool::<RingPool<T>>(true, amount)?;

			Ok(())
		}

		fn stake_kton(ledger: &mut Ledger<T>, amount: Balance) -> DispatchResult {
			T::Kton::stake(&ledger.account, amount)?;

			ledger.staked_kton = if let Some(k) = ledger.staked_kton.checked_add(amount) {
				k
			} else {
				return Err("[pallet::staking] `u128` must not be overflowed; qed")?;
			};

			Self::update_pool::<KtonPool<T>>(true, amount)?;

			Ok(())
		}

		fn stake_deposit(ledger: &mut Ledger<T>, deposit: DepositId<T>) -> DispatchResult {
			T::Deposit::stake(&ledger.account, deposit)?;

			ledger.staked_deposits.try_push(deposit).map_err(|_| <Error<T>>::ExceedMaxDeposits)?;

			Self::update_pool::<RingPool<T>>(true, T::Deposit::amount(&ledger.account, deposit))?;

			Ok(())
		}

		fn unstake_ring(ledger: &mut Ledger<T>, amount: Balance) -> DispatchResult {
			let Some(nr) = ledger.staked_ring.checked_sub(amount) else {
				return Err("[pallet::staking] `u128` must not be overflowed; qed")?;
			};

			ledger.staked_ring = nr;
			ledger
				.unstaking_ring
				.try_push((
					amount,
					<frame_system::Pallet<T>>::block_number() + T::MinStakingDuration::get(),
				))
				.map_err(|_| <Error<T>>::ExceedMaxUnstakings)?;

			Self::update_pool::<RingPool<T>>(false, amount)?;

			Ok(())
		}

		fn unstake_kton(ledger: &mut Ledger<T>, amount: Balance) -> DispatchResult {
			let Some(nk) = ledger.staked_kton.checked_sub(amount) else {
				return Err("[pallet::staking] `u128` must not be overflowed; qed")?;
			};

			ledger.staked_kton = nk;
			ledger
				.unstaking_kton
				.try_push((
					amount,
					<frame_system::Pallet<T>>::block_number() + T::MinStakingDuration::get(),
				))
				.map_err(|_| <Error<T>>::ExceedMaxUnstakings)?;

			Self::update_pool::<KtonPool<T>>(false, amount)?;

			Ok(())
		}

		fn unstake_deposit(ledger: &mut Ledger<T>, deposit: DepositId<T>) -> DispatchResult {
			let Some(i) = ledger.staked_deposits.iter().position(|d| d == &deposit) else {
				return Err("[pallet::staking] deposit id must be existed, due to previous unstake OP; qed")?;
			};

			ledger
				.unstaking_deposits
				.try_push((
					ledger.staked_deposits.remove(i),
					<frame_system::Pallet<T>>::block_number() + T::MinStakingDuration::get(),
				))
				.map_err(|_| <Error<T>>::ExceedMaxUnstakings)?;

			Self::update_pool::<RingPool<T>>(false, T::Deposit::amount(&ledger.account, deposit))?;

			Ok(())
		}

		fn claim_unstakings(who: &T::AccountId) -> DispatchResult {
			<Ledgers<T>>::try_mutate(who, |l| {
				let Some(l) = l else {
					return Err(<Error<T>>::NotStaker)?;
				};
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
				T::Ring::unstake(who, r_claimed)?;

				let mut k_claimed = 0;

				claim(&mut l.unstaking_kton, &mut k_claimed);
				T::Kton::unstake(who, k_claimed)?;

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
				let Some(l) = maybe_l else { return Err(()); };

				if l.is_empty() {
					*maybe_l = None;

					<frame_system::Pallet<T>>::dec_consumers(who);

					Ok(())
				} else {
					Err(())
				}
			});
		}

		fn ensure_staker(who: &T::AccountId) -> DispatchResult {
			if <Ledgers<T>>::contains_key(who) {
				Ok(())
			} else {
				Err(<Error<T>>::NotStaker)?
			}
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
								.fold(0, |r, d| r + T::Deposit::amount(who, d)),
					) + Self::balance2power::<KtonPool<T>>(l.staked_kton)
				})
				.unwrap_or_default()
		}

		// TODO: weight
		/// Pay the session reward to the stakers.
		pub fn payout(session_duration: Moment, elapsed_time: Moment) {
			let unminted = TOTAL_SUPPLY - T::RingCurrency::total_issuance();

			log::info!(
				"\
					[pallet::staking] making a payout for: \
					`unminted = {unminted}`, \
					`session_duration = {session_duration}`, \
					`elapsed_time = {elapsed_time}`\
				"
			);

			let Some(inflation) = dc_inflation::in_period(
				unminted,
				session_duration,
				elapsed_time,
			) else {
				return;
			};
			let payout = T::PayoutFraction::get() * inflation;
			let (total_points, reward_map) = <RewardPoints<T>>::get();
			// Due to the `payout * percent` there might be some losses.
			let mut actual_payout = 0;

			for (c, p) in reward_map {
				let Some(commission) = <Collators<T>>::get(&c) else {
					log::error!("[pallet::staking] collator({c:?}) must be found; qed");

					continue;
				};
				let c_total_payout = Perbill::from_rational(p, total_points) * payout;
				let c_commission_payout = commission * c_total_payout;
				let n_payout = c_total_payout - c_commission_payout;
				let Some(c_exposure) = <Exposures<T>>::get(&c) else {
					log::error!("[pallet::staking] exposure({c:?}) must be found; qed");

					continue;
				};
				let c_payout = c_commission_payout
					+ Perbill::from_rational(c_exposure.own, c_exposure.total) * n_payout;

				if let Ok(_i) = T::RingCurrency::deposit_into_existing(&c, c_payout) {
					actual_payout += c_payout;

					Self::deposit_event(Event::<T>::Payout { staker: c, ring_amount: c_payout });
				}

				for n_exposure in c_exposure.others {
					let n_payout =
						Perbill::from_rational(n_exposure.value, c_exposure.total) * n_payout;

					if let Ok(_i) =
						T::RingCurrency::deposit_into_existing(&n_exposure.who, n_payout)
					{
						actual_payout += n_payout;

						Self::deposit_event(Event::<T>::Payout {
							staker: n_exposure.who,
							ring_amount: n_payout,
						});
					}
				}
			}

			T::RewardRemainder::on_unbalanced(T::RingCurrency::issue(inflation - actual_payout));
		}

		/// Clean the old session data.
		pub fn clean_old_session() {
			<RewardPoints<T>>::kill();
			#[allow(deprecated)]
			<Exposures<T>>::remove_all(None);
		}

		/// Elect the new collators.
		///
		/// This should only be called by the [`pallet_session::SessionManager::new_session`].
		pub fn elect() -> Vec<T::AccountId> {
			let mut collators = <Collators<T>>::iter_keys()
				.map(|c| {
					let c_power = Self::power_of(&c);
					let mut t_power = c_power;
					let i_exposures = <Nominators<T>>::iter()
						.filter_map(|(n, c_)| {
							if c_ == c {
								let n_power = Self::power_of(&n);

								t_power += n_power;

								Some(IndividualExposure { who: n, value: n_power })
							} else {
								None
							}
						})
						.collect();

					((c, Exposure { total: t_power, own: c_power, others: i_exposures }), t_power)
				})
				.collect::<Vec<_>>();

			collators.sort_by(|(_, a), (_, b)| b.cmp(a));

			collators
				.into_iter()
				.take(<CollatorCount<T>>::get() as _)
				.map(|((c, e), _)| {
					<Exposures<T>>::insert(&c, e);

					c
				})
				.collect()
		}
	}
}
pub use pallet::*;

// Add reward points to block authors:
// - 20 points to the block producer for producing a (non-uncle) block in the parachain chain,
// - 2 points to the block producer for each reference to a previously unreferenced uncle, and
// - 1 point to the producer of each referenced uncle block.
impl<T> pallet_authorship::EventHandler<T::AccountId, T::BlockNumber> for Pallet<T>
where
	T: Config + pallet_authorship::Config + pallet_session::Config,
{
	fn note_author(author: T::AccountId) {
		Self::reward_by_ids(&[(author, 20)])
	}

	fn note_uncle(uncle_author: T::AccountId, _age: T::BlockNumber) {
		if let Some(block_author) = <pallet_authorship::Pallet<T>>::author() {
			Self::reward_by_ids(&[(block_author, 2), (uncle_author, 1)])
		} else {
			log::error!("[pallet::staking] block author not set, this should never happen; qed");
		}
	}
}

// Play the role of the session manager.
impl<T> pallet_session::SessionManager<T::AccountId> for Pallet<T>
where
	T: Config,
{
	fn new_session(index: u32) -> Option<Vec<T::AccountId>> {
		log::info!(
			"[pallet::staking] assembling new collators for new session {} at #{:?}",
			index,
			<frame_system::Pallet<T>>::block_number(),
		);

		let collators = Self::elect();

		Self::deposit_event(Event::<T>::Elected { collators: collators.clone() });

		Some(collators)
	}

	fn start_session(_: u32) {}

	fn end_session(_: u32) {
		let now = T::UnixTime::now().as_millis();
		let session_duration = now - <SessionStartTime<T>>::get();
		let elapsed_time = <ElapsedTime<T>>::mutate(|t| {
			*t += session_duration;

			*t
		});

		<SessionStartTime<T>>::put(now);

		Self::payout(session_duration, elapsed_time);
		Self::clean_old_session();
	}
}

/// The account of the staking pot.
pub fn account_id<A>() -> A
where
	A: FullCodec,
{
	PalletId(*b"dar/stak").into_account_truncating()
}
