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

mod weights;
pub use weights::WeightInfo;

// core
use core::{
	cmp::Ordering::{Equal, Greater, Less},
	ops::ControlFlow::{Break, Continue},
};
// crates.io
use codec::FullCodec;
// darwinia
use dc_inflation::MILLISECS_PER_YEAR;
use dc_types::{Balance, Moment};
// substrate
use frame_support::{
	pallet_prelude::*,
	traits::{
		Currency,
		ExistenceRequirement::{AllowDeath, KeepAlive},
		UnixTime,
	},
	PalletId,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::AccountIdConversion;

/// Milliseconds per month.
pub const MILLISECS_PER_MONTH: Moment = MILLISECS_PER_YEAR / 12;

/// Simple asset APIs.
pub trait SimpleAsset {
	/// Account type.
	type AccountId;

	/// Mint API.
	fn mint(beneficiary: &Self::AccountId, amount: Balance) -> DispatchResult;

	/// Burn API.
	fn burn(who: &Self::AccountId, amount: Balance) -> DispatchResult;
}

/// Deposit identifier.
///
/// It's not a global-unique identifier.
/// It's only used for distinguishing the deposits under a specific account.
// https://github.com/polkadot-js/apps/issues/8591
// pub type DepositId = u8;
pub type DepositId = u16;

/// Deposit.
#[derive(PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
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

		/// RING asset.
		type Ring: Currency<Self::AccountId, Balance = Balance>;

		/// KTON asset.
		type Kton: SimpleAsset<AccountId = Self::AccountId>;

		/// Minimum amount to lock at least.
		#[pallet::constant]
		type MinLockingAmount: Get<Balance>;

		/// Maximum deposit count.
		///
		/// In currently design, this should not be greater than `u8::MAX`.
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
		/// An expired deposit has been claimed.
		DepositClaimed { owner: T::AccountId, deposit_id: DepositId },
		/// An unexpired deposit has been claimed by paying the KTON penalty.
		DepositClaimedWithPenalty {
			owner: T::AccountId,
			deposit_id: DepositId,
			kton_penalty: Balance,
		},
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

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Lock the RING for some KTON profit/interest.
		#[pallet::weight(0)]
		pub fn lock(origin: OriginFor<T>, amount: Balance, months: u8) -> DispatchResult {
			let who = ensure_signed(origin)?;

			if amount < T::MinLockingAmount::get() {
				Err(<Error<T>>::LockAtLeastSome)?;
			}
			if months == 0 {
				Err(<Error<T>>::LockAtLeastOneMonth)?;
			}
			if months > 36 {
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
					Less => Break(i),
					Equal => Continue(i + 1),
					Greater => Break(i - 1),
				}) {
					Continue(c) => c,
					Break(b) => b,
				};
				let start_time = T::UnixTime::now().as_millis();
				let expired_time = start_time + MILLISECS_PER_MONTH * months as Moment;

				ds.try_insert(
					id as _,
					Deposit { id, value: amount, start_time, expired_time, in_use: false },
				)
				.map_err(|_| <Error<T>>::ExceedMaxDeposits)?;

				<Result<_, DispatchError>>::Ok((id, start_time, expired_time))
			})?;

			T::Ring::transfer(&who, &account_id(), amount, KeepAlive)?;

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
		#[pallet::weight(0)]
		pub fn claim(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let now = T::UnixTime::now().as_millis();
			let mut claimed = 0;
			let _ = <Deposits<T>>::try_mutate(&who, |maybe_ds| {
				let ds = maybe_ds.as_mut().ok_or(())?;

				ds.retain(|d| {
					if d.expired_time <= now && !d.in_use {
						claimed += d.value;

						Self::deposit_event(Event::DepositClaimed {
							owner: who.clone(),
							deposit_id: d.id,
						});

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

			T::Ring::transfer(&account_id(), &who, claimed, AllowDeath)?;

			Ok(())
		}

		/// Claim the unexpired-locked RING by paying the KTON penalty.
		#[pallet::weight(0)]
		pub fn claim_with_penalty(origin: OriginFor<T>, id: DepositId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let d = <Deposits<T>>::try_mutate(&who, |maybe_ds| {
				let ds = maybe_ds.as_mut().ok_or(<Error<T>>::DepositNotFound)?;
				let d = ds
					.remove(ds.iter().position(|d| d.id == id).ok_or(<Error<T>>::DepositNotFound)?);

				if ds.is_empty() {
					<frame_system::Pallet<T>>::dec_consumers(&who);

					*maybe_ds = None;
				}

				<Result<_, DispatchError>>::Ok(d)
			})?;
			let now = T::UnixTime::now().as_millis();

			if d.expired_time <= now {
				Err(<Error<T>>::DepositAlreadyExpired)?;
			}

			let promise_m = (d.expired_time - d.start_time) / MILLISECS_PER_MONTH;
			let elapsed_m = (now - d.start_time) / MILLISECS_PER_MONTH;
			let kton_penalty = dc_inflation::deposit_interest(d.value, promise_m as _)
				.saturating_sub(dc_inflation::deposit_interest(d.value, elapsed_m as _))
				.max(1) * 3;

			T::Kton::burn(&who, kton_penalty)?;
			Self::deposit_event(Event::DepositClaimedWithPenalty {
				owner: who,
				deposit_id: id,
				kton_penalty,
			});

			Ok(())
		}
	}
}
pub use pallet::*;

impl<T> darwinia_staking::Stake for Pallet<T>
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
impl<T> darwinia_staking::StakeExt for Pallet<T>
where
	T: Config,
{
	type Amount = Balance;

	fn amount(who: &Self::AccountId, item: Self::Item) -> Self::Amount {
		<Deposits<T>>::get(who)
			.and_then(|ds| {
				ds.into_iter().find_map(|d| if d.id == item { Some(d.value) } else { None })
			})
			.unwrap_or_default()
	}
}

/// The account of the deposit pot.
pub fn account_id<A>() -> A
where
	A: FullCodec,
{
	PalletId(*b"dar/depo").into_account_truncating()
}
