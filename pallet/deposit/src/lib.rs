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
// Darwinia is distributed in_use the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

//! # Darwinia deposit pallet
//!
//! ## Overview
//!
//! This is a completely specialized deposit pallet designed only for Darwinia parachain.
//! So, this pallet will eliminate the generic parameters as much as possible.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;
pub use weights::WeightInfo;

// core
use core::{
	cmp::Ordering::{Equal, Greater, Less},
	marker::PhantomData,
	ops::ControlFlow::{Break, Continue},
};
// crates.io
use codec::FullCodec;
use ethabi::{Function, Param, ParamType, StateMutability, Token};
// darwinia
use dc_inflation::MILLISECS_PER_YEAR;
use dc_types::{Balance, Moment};
// polkadot-sdk
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement::AllowDeath, UnixTime},
	PalletId,
};
use frame_system::pallet_prelude::*;
use sp_core::H160;
use sp_runtime::traits::AccountIdConversion;

#[frame_support::pallet]
pub mod pallet {
	// darwinia
	use crate::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_timestamp::Config {
		/// Override the [`frame_system::Config::RuntimeEvent`].
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// RING asset.
		type Ring: Currency<Self::AccountId, Balance = Balance>;

		/// KTON asset.
		type Kton: SimpleAsset<AccountId = Self::AccountId>;

		/// Deposit contract migrator.
		type DepositMigrator: MigrateToContract<Self>;

		/// Treasury account.
		#[pallet::constant]
		type Treasury: Get<Self::AccountId>;

		/// Minimum amount to lock at least.
		#[pallet::constant]
		type MinLockingAmount: Get<Balance>;

		/// Maximum deposit count.
		///
		/// In currently design, this should not be greater than `u16::MAX`.
		#[pallet::constant]
		type MaxDeposits: Get<u32>;
	}

	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new deposit has been created.
		DepositCreated {
			owner: T::AccountId,
			deposit_id: DepositId,
			value: Balance,
			start_time: Moment,
			expired_time: Moment,
			kton_reward: Balance,
		},
		/// Expired deposits have been claimed.
		DepositsClaimed { owner: T::AccountId, deposits: Vec<DepositId> },
		/// An unexpired deposit has been claimed by paying the KTON penalty.
		DepositClaimedWithPenalty {
			owner: T::AccountId,
			deposit_id: DepositId,
			kton_penalty: Balance,
		},
		/// Deposits have been migrated.
		DepositsMigrated { owner: T::AccountId, deposits: Vec<DepositId> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Lock at least for a specific amount.
		LockAtLeastSome,
		/// Lock at least for one month.
		LockAtLeastOneMonth,
		/// Lock at most for thirty-six months.
		LockAtMostThirtySixMonths,
		/// Exceed maximum deposit count.
		ExceedMaxDeposits,
		/// Deposit not found.
		DepositNotFound,
		/// Deposit is in use.
		DepositInUse,
		/// Deposit is not in use.
		DepositNotInUse,
		/// Deposit is already expired.
		DepositAlreadyExpired,
	}

	/// All deposits.
	///
	/// The items must be sorted by the id.
	#[pallet::storage]
	#[pallet::getter(fn deposit_of)]
	pub type Deposits<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<Deposit, T::MaxDeposits>>;

	// Deposit contract address.
	#[pallet::storage]
	#[pallet::getter(fn deposit_contract)]
	pub type DepositContract<T: Config> = StorageValue<_, T::AccountId>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Lock the RING for some KTON profit/interest.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::lock())]
		pub fn lock(origin: OriginFor<T>, amount: Balance, months: u8) -> DispatchResult {
			let who = ensure_signed(origin)?;

			if amount < T::MinLockingAmount::get() {
				Err(<Error<T>>::LockAtLeastSome)?;
			}
			if months == 0 {
				Err(<Error<T>>::LockAtLeastOneMonth)?;
			}
			if months > MAX_LOCKING_MONTHS {
				Err(<Error<T>>::LockAtMostThirtySixMonths)?;
			}
			if <Deposits<T>>::decode_len(&who).unwrap_or_default() as u32 >= T::MaxDeposits::get() {
				Err(<Error<T>>::ExceedMaxDeposits)?;
			}

			let (deposit_id, start_time, expired_time) = <Deposits<T>>::try_mutate(&who, |ds| {
				let ds = if let Some(ds) = ds {
					ds
				} else {
					<frame_system::Pallet<T>>::inc_consumers(&who)?;

					*ds = Some(Default::default());

					ds.as_mut().expect("[pallet::deposit] `ds` must be some; qed")
				};

				// Keep the list sorted in increasing order.
				// And find the missing id.
				let id = match ds.iter().map(|d| d.id).try_fold(0, |i, id| match i.cmp(&id) {
					// `Greater` never occurs, as this list is always sorted.
					// But it's a good practice to handle it instead of marking as `unreachable!`.
					Less | Greater => Break(i),
					Equal => Continue(i + 1),
				}) {
					Continue(c) => c,
					Break(b) => b,
				};
				let start_time = Self::now();
				let expired_time = start_time + MILLISECS_PER_MONTH * months as Moment;

				ds.try_insert(
					id as _,
					Deposit { id, value: amount, start_time, expired_time, in_use: false },
				)
				.map_err(|_| <Error<T>>::ExceedMaxDeposits)?;

				<Result<_, DispatchError>>::Ok((id, start_time, expired_time))
			})?;

			T::Ring::transfer(&who, &account_id(), amount, AllowDeath)?;

			let kton_reward = dc_inflation::deposit_interest(amount, months);

			T::Kton::mint(&who, kton_reward)?;

			Self::deposit_event(Event::DepositCreated {
				owner: who,
				deposit_id,
				value: amount,
				start_time,
				expired_time,
				kton_reward,
			});

			Ok(())
		}

		/// Claim the expired-locked RING.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::claim())]
		pub fn claim(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let now = Self::now();
			let mut to_claim = (0, Vec::new());
			let _ = <Deposits<T>>::try_mutate(&who, |maybe_ds| {
				let ds = maybe_ds.as_mut().ok_or(())?;

				ds.retain(|d| {
					if d.expired_time <= now && !d.in_use {
						to_claim.0 += d.value;
						to_claim.1.push(d.id);

						false
					} else {
						true
					}
				});

				if ds.is_empty() {
					<frame_system::Pallet<T>>::dec_consumers(&who);

					*maybe_ds = None;
				}

				<Result<(), ()>>::Ok(())
			});

			T::Ring::transfer(&account_id(), &who, to_claim.0, AllowDeath)?;
			Self::deposit_event(Event::DepositsClaimed { owner: who, deposits: to_claim.1 });

			Ok(())
		}

		/// Claim the unexpired-locked RING by paying the KTON penalty.
		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::claim_with_penalty())]
		pub fn claim_with_penalty(origin: OriginFor<T>, id: DepositId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let d = <Deposits<T>>::try_mutate(&who, |maybe_ds| {
				let ds = maybe_ds.as_mut().ok_or(<Error<T>>::DepositNotFound)?;
				let d = ds
					.remove(ds.iter().position(|d| d.id == id).ok_or(<Error<T>>::DepositNotFound)?);

				if d.in_use {
					Err(<Error<T>>::DepositInUse)?;
				}
				if ds.is_empty() {
					<frame_system::Pallet<T>>::dec_consumers(&who);

					*maybe_ds = None;
				}

				Ok::<_, DispatchError>(d)
			})?;
			let now = Self::now();

			if d.expired_time <= now {
				Err(<Error<T>>::DepositAlreadyExpired)?;
			}

			let promise_m = (d.expired_time - d.start_time) / MILLISECS_PER_MONTH;
			let elapsed_m = (now - d.start_time) / MILLISECS_PER_MONTH;
			let kton_penalty = dc_inflation::deposit_interest(d.value, promise_m as _)
				.saturating_sub(dc_inflation::deposit_interest(d.value, elapsed_m as _))
				.max(1) * 3;

			T::Kton::burn(&who, kton_penalty)?;
			T::Ring::transfer(&account_id(), &who, d.value, AllowDeath)?;
			Self::deposit_event(Event::DepositClaimedWithPenalty {
				owner: who,
				deposit_id: id,
				kton_penalty,
			});

			Ok(())
		}

		/// Migration data to deposit contract.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::migrate())]
		pub fn migrate(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let Some(ds) = <Deposits<T>>::take(&who) else { return Ok(()) };
			let now = Self::now();

			for c in ds.chunks(50) {
				let mut to_claim = (0, Vec::new());
				let mut to_migrate = (0, Vec::new(), Vec::new());

				for d in c {
					if d.in_use {
						Err(<Error<T>>::DepositInUse)?;
					}

					if d.expired_time <= now {
						to_claim.0 += d.value;
						to_claim.1.push(d.id);
					} else {
						to_migrate.0 += d.value;
						to_migrate.1.push(d.id);
						to_migrate.2.push((d.value, d.start_time, d.expired_time));
					}
				}

				T::Ring::transfer(&account_id(), &who, to_claim.0, AllowDeath)?;
				T::Ring::transfer(&account_id(), &T::Treasury::get(), to_migrate.0, AllowDeath)?;
				T::DepositMigrator::migrate(who.clone(), to_migrate.2);

				Self::deposit_event(Event::DepositsClaimed {
					owner: who.clone(),
					deposits: to_claim.1,
				});
				Self::deposit_event(Event::DepositsMigrated {
					owner: who.clone(),
					deposits: to_migrate.1,
				});
			}

			Ok(())
		}

		/// Set deposit contract address.
		#[pallet::call_index(4)]
		#[pallet::weight(<T as Config>::WeightInfo::set_deposit_contract())]
		pub fn set_deposit_contract(
			origin: OriginFor<T>,
			deposit_contract: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;

			<DepositContract<T>>::put(deposit_contract);

			Ok(())
		}
	}
	impl<T> Pallet<T>
	where
		T: Config,
	{
		fn now() -> Moment {
			<pallet_timestamp::Pallet<T> as UnixTime>::now().as_millis()
		}
	}
	impl<T> darwinia_staking_traits::Stake for Pallet<T>
	where
		T: Config,
	{
		type AccountId = T::AccountId;
		type Item = DepositId;

		fn stake(who: &Self::AccountId, item: Self::Item) -> DispatchResult {
			<Deposits<T>>::try_mutate(who, |ds| {
				let ds = ds.as_mut().ok_or(<Error<T>>::DepositNotFound)?;
				let d = ds.iter_mut().find(|d| d.id == item).ok_or(<Error<T>>::DepositNotFound)?;

				if d.in_use {
					Err(<Error<T>>::DepositInUse)?
				} else {
					d.in_use = true;

					Ok(())
				}
			})
		}

		fn unstake(who: &Self::AccountId, item: Self::Item) -> DispatchResult {
			<Deposits<T>>::try_mutate(who, |ds| {
				let ds = ds.as_mut().ok_or(<Error<T>>::DepositNotFound)?;
				let d = ds.iter_mut().find(|d| d.id == item).ok_or(<Error<T>>::DepositNotFound)?;

				if d.in_use {
					d.in_use = false;

					Ok(())
				} else {
					Err(<Error<T>>::DepositNotInUse)?
				}
			})
		}
	}
	impl<T> darwinia_staking_traits::StakeExt for Pallet<T>
	where
		T: Config,
	{
		type Amount = Balance;

		fn amount(who: &Self::AccountId, item: Self::Item) -> Result<Self::Amount, DispatchError> {
			Ok(<Deposits<T>>::get(who)
				.and_then(|ds| {
					ds.into_iter().find_map(|d| if d.id == item { Some(d.value) } else { None })
				})
				.ok_or(<Error<T>>::DepositNotFound)?)
		}
	}
}
pub use pallet::*;

/// Deposit identifier.
pub type DepositId = u16;
// https://github.com/polkadot-js/apps/issues/8591
// Max deposits in Darwinia is 322.
// Max deposits in Crab is 220.
// Maybe we will use `WeakBoundedVec` later.
// pub type DepositId = u8;

/// Milliseconds per month.
pub const MILLISECS_PER_MONTH: Moment = MILLISECS_PER_YEAR / 12;

/// The maximum locking period for a deposit.
pub const MAX_LOCKING_MONTHS: u8 = 36;

/// Simple asset APIs.
pub trait SimpleAsset {
	/// Account type.
	type AccountId;

	/// Mint API.
	fn mint(beneficiary: &Self::AccountId, amount: Balance) -> DispatchResult;

	/// Burn API.
	fn burn(who: &Self::AccountId, amount: Balance) -> DispatchResult;
}

/// Migrate to contract trait.
pub trait MigrateToContract<T>
where
	T: Config,
{
	/// Migrate to contract.
	fn migrate(_: T::AccountId, _: Vec<(Balance, Moment, Moment)>) {}
}
impl<T> MigrateToContract<T> for () where T: Config {}

/// Deposit.
#[derive(Clone, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
pub struct Deposit {
	/// Deposit ID.
	pub id: DepositId,
	/// Deposited RING.
	pub value: Balance,
	/// Start timestamp.
	pub start_time: Moment,
	/// Expired timestamp.
	pub expired_time: Moment,
	/// Deposit state.
	pub in_use: bool,
}

/// Deposit migrator.
pub struct DepositMigrator<T>(PhantomData<T>);
impl<T> MigrateToContract<T> for DepositMigrator<T>
where
	T: Config + darwinia_ethtx_forwarder::Config,
	T::AccountId: Into<H160>,
{
	fn migrate(who: T::AccountId, deposits: Vec<(Balance, Moment, Moment)>) {
		let Some(dc) = <DepositContract<T>>::get() else {
			log::error!("deposit contract must be some; qed");

			return;
		};
		let dc = dc.into();

		#[allow(deprecated)]
		darwinia_ethtx_forwarder::quick_forward_transact::<T>(
			T::Treasury::get().into(),
			Function {
				name: "migrate".into(),
				inputs: vec![
					Param {
						name: "address".to_owned(),
						kind: ParamType::Address,
						internal_type: None,
					},
					Param {
						name: "deposits".to_owned(),
						kind: ParamType::Array(Box::new(ParamType::Tuple(vec![
							ParamType::Uint(128),
							ParamType::Uint(64),
							ParamType::Uint(64),
						]))),
						internal_type: None,
					},
				],
				outputs: Vec::new(),
				constant: None,
				state_mutability: StateMutability::Payable,
			},
			&[
				Token::Address(who.into()),
				Token::Array(
					deposits
						.into_iter()
						.map(|(v, s, e)| {
							Token::Tuple(vec![
								Token::Uint(v.into()),
								Token::Uint(s.into()),
								Token::Uint(e.into()),
							])
						})
						.collect(),
				),
			],
			dc,
			0.into(),
			1_000_000.into(),
		)
	}
}

/// The account of the deposit pot.
pub fn account_id<A>() -> A
where
	A: FullCodec,
{
	PalletId(*b"dar/depo").into_account_truncating()
}
