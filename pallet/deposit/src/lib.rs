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
use core::marker::PhantomData;
// crates.io
use codec::FullCodec;
use ethabi::{Function, Param, ParamType, StateMutability, Token};
// darwinia
use dc_types::{Balance, Moment};
// frontier
use fp_evm::{CallOrCreateInfo, ExitReason};
// polkadot-sdk
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement::AllowDeath, UnixTime},
	PalletId,
};
use frame_system::pallet_prelude::*;
use sp_core::H160;
use sp_runtime::traits::AccountIdConversion;
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

#[frame_support::pallet]
pub mod pallet {
	// darwinia
	use crate::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_timestamp::Config {
		/// Override the [`frame_system::Config::RuntimeEvent`].
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsic in this pallet.
		type WeightInfo: WeightInfo;

		/// RING asset.
		type Ring: Currency<Self::AccountId, Balance = Balance>;

		/// Deposit contract migrator.
		type DepositMigrator: MigrateToContract<Self>;

		/// Treasury account.
		#[pallet::constant]
		type Treasury: Get<Self::AccountId>;
	}

	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Expired deposits have been claimed.
		DepositsClaimed { owner: T::AccountId, deposits: Vec<DepositId> },
		/// Deposits have been migrated.
		DepositsMigrated { owner: T::AccountId, deposits: Vec<DepositId> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// No deposit.
		NoDeposit,
		/// Invalid deposit contract.
		InvalidDepositContract,
		/// Migration interaction with deposit contract failed.
		MigrationFailedOnContract,
	}

	/// All deposits.
	///
	/// The items must be sorted by the id.
	#[pallet::storage]
	#[pallet::getter(fn deposit_of)]
	pub type Deposits<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<Deposit, ConstU32<512>>>;

	// Deposit contract address.
	#[pallet::storage]
	#[pallet::getter(fn deposit_contract)]
	pub type DepositContract<T: Config> = StorageValue<_, T::AccountId>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_idle(_: BlockNumberFor<T>, mut remaining_weight: Weight) -> Weight {
			// At least 1 read weight is required.
			#[cfg(not(test))]
			if let Some(rw) = remaining_weight.checked_sub(&T::DbWeight::get().reads(1)) {
				remaining_weight = rw;
			} else {
				return remaining_weight;
			}

			#[cfg(test)]
			let wt = Weight::zero().add_ref_time(10);
			#[cfg(not(test))]
			let wt = <T as Config>::WeightInfo::migrate_for();
			let mut ds_to_migrate = BTreeMap::<T::AccountId, (Vec<Deposit>, usize)>::new();

			'outer: for (w, ds) in <Deposits<T>>::iter() {
				for _ in 0..ds.len().div_ceil(10) {
					if let Some(rw) = remaining_weight.checked_sub(&wt) {
						remaining_weight = rw;

						if let Some((_, cnt)) = ds_to_migrate.get_mut(&w) {
							*cnt += 1;
						} else {
							ds_to_migrate.insert(w.clone(), (ds.to_vec(), 1));
						}
					} else {
						break 'outer;
					}
				}
			}

			for (w, (ds, cnt)) in ds_to_migrate {
				let mut ds = ds;

				for _ in 0..cnt {
					match Self::migrate_for_inner(&w, ds.clone()) {
						Ok(ds_) => ds = ds_,
						_ => break,
					}
				}

				if ds.is_empty() {
					<Deposits<T>>::remove(&w);
				} else {
					// There are still some deposits left for this account.
					<Deposits<T>>::insert(&w, BoundedVec::truncate_from(ds));
				}
			}

			remaining_weight
		}
	}
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Migrate the specified account's data to deposit contract.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::migrate_for())]
		pub fn migrate_for(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			ensure_signed(origin)?;

			let ds = <Deposits<T>>::take(&who).ok_or(<Error<T>>::NoDeposit)?;
			let ds = Self::migrate_for_inner(&who, ds)?;

			// Put the rest deposits back.
			if !ds.is_empty() {
				<Deposits<T>>::insert(&who, BoundedVec::truncate_from(ds));
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

		fn migrate_for_inner<I>(
			who: &T::AccountId,
			deposits: I,
		) -> Result<Vec<Deposit>, DispatchError>
		where
			I: IntoIterator<Item = Deposit>,
		{
			let now = Self::now();
			let mut deposits = deposits.into_iter();
			let mut to_claim = (0, Vec::new());
			let mut to_migrate = (0, Vec::new(), Vec::new());

			// Take 0~10 deposits to migrate.
			for d in deposits.by_ref().take(10) {
				if d.expired_time <= now {
					to_claim.0 += d.value;
					to_claim.1.push(d.id);
				} else {
					to_migrate.0 += d.value;
					to_migrate.1.push(d.id);
					to_migrate.2.push((d.value, d.start_time / 1_000, d.expired_time / 1_000));
				}
			}

			T::Ring::transfer(&account_id(), who, to_claim.0, AllowDeath)?;
			T::Ring::transfer(&account_id(), &T::Treasury::get(), to_migrate.0, AllowDeath)?;
			T::DepositMigrator::migrate(who.clone(), to_migrate.0, to_migrate.2)?;

			Self::deposit_event(Event::DepositsClaimed {
				owner: who.clone(),
				deposits: to_claim.1,
			});
			Self::deposit_event(Event::DepositsMigrated {
				owner: who.clone(),
				deposits: to_migrate.1,
			});

			Ok(deposits.collect())
		}
	}
}
pub use pallet::*;

/// Deposit identifier.
pub type DepositId = u16;

/// Migrate to contract trait.
pub trait MigrateToContract<T>
where
	T: Config,
{
	/// Migrate to contract.
	fn migrate(_: T::AccountId, _: Balance, _: Vec<(Balance, Moment, Moment)>) -> DispatchResult {
		Ok(())
	}
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
	fn migrate(
		who: T::AccountId,
		total: Balance,
		deposits: Vec<(Balance, Moment, Moment)>,
	) -> DispatchResult {
		let cnt = deposits.len();
		let dc = <DepositContract<T>>::get().ok_or(<Error<T>>::InvalidDepositContract)?.into();
		#[allow(deprecated)]
		let exit_reason = match darwinia_ethtx_forwarder::quick_forward_transact::<T>(
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
			total.into(),
			// Approximately consume 160,000 gas per deposit on Koi testnet.
			(200_000 * cnt as u64).into(),
		)?
		.1
		{
			CallOrCreateInfo::Call(i) => i.exit_reason,
			CallOrCreateInfo::Create(i) => i.exit_reason,
		};

		match exit_reason {
			ExitReason::Succeed(_) => Ok(()),
			_ => Err(<Error<T>>::MigrationFailedOnContract)?,
		}
	}
}

/// The account of the deposit pot.
pub fn account_id<A>() -> A
where
	A: FullCodec,
{
	PalletId(*b"dar/depo").into_account_truncating()
}
