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

//! Autogenerated weights for `darwinia_account_migration`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-12-05, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `Xaviers-MacBook-Pro-16.local`, CPU: `<UNKNOWN>`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("darwinia-dev")`, DB CACHE: 1024

// Executed Command:
// target/release/darwinia
// benchmark
// pallet
// --header
// .maintain/license-header
// --heap-pages
// 4096
// --chain
// darwinia-dev
// --output
// runtime/darwinia/src/weights
// --pallet
// *
// --extrinsic
// *
// --steps
// 50
// --repeat
// 20

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `darwinia_account_migration`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> darwinia_account_migration::WeightInfo for WeightInfo<T> {
	/// Storage: `AccountMigration::Accounts` (r:1 w:1)
	/// Proof: `AccountMigration::Accounts` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `AccountMigration::KtonAccounts` (r:1 w:1)
	/// Proof: `AccountMigration::KtonAccounts` (`max_values`: None, `max_size`: Some(82), added: 2557, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Asset` (r:1 w:1)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(166), added: 2641, mode: `MaxEncodedLen`)
	/// Storage: `AccountMigration::Identities` (r:1 w:1)
	/// Proof: `AccountMigration::Identities` (`max_values`: None, `max_size`: Some(9219), added: 11694, mode: `MaxEncodedLen`)
	/// Storage: `Identity::Registrars` (r:1 w:1)
	/// Proof: `Identity::Registrars` (`max_values`: Some(1), `max_size`: Some(901), added: 1396, mode: `MaxEncodedLen`)
	/// Storage: `AccountMigration::Ledgers` (r:1 w:1)
	/// Proof: `AccountMigration::Ledgers` (`max_values`: None, `max_size`: Some(1845), added: 4320, mode: `MaxEncodedLen`)
	/// Storage: `AccountMigration::Deposits` (r:1 w:1)
	/// Proof: `AccountMigration::Deposits` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:2 w:3)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(116), added: 2591, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Account` (r:1 w:2)
	/// Proof: `Assets::Account` (`max_values`: None, `max_size`: Some(114), added: 2589, mode: `MaxEncodedLen`)
	/// Storage: `DarwiniaStaking::Ledgers` (r:0 w:1)
	/// Proof: `DarwiniaStaking::Ledgers` (`max_values`: None, `max_size`: Some(1833), added: 4308, mode: `MaxEncodedLen`)
	/// Storage: `Identity::IdentityOf` (r:0 w:1)
	/// Proof: `Identity::IdentityOf` (`max_values`: None, `max_size`: Some(7526), added: 10001, mode: `MaxEncodedLen`)
	/// Storage: `Deposit::Deposits` (r:0 w:1)
	/// Proof: `Deposit::Deposits` (`max_values`: None, `max_size`: Some(26150), added: 28625, mode: `MaxEncodedLen`)
	fn migrate() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `29175`
		//  Estimated: `32640`
		// Minimum execution time: 202_000_000 picoseconds.
		Weight::from_parts(205_000_000, 0)
			.saturating_add(Weight::from_parts(0, 32640))
			.saturating_add(T::DbWeight::get().reads(10))
			.saturating_add(T::DbWeight::get().writes(15))
	}
	/// Storage: `AccountMigration::Multisigs` (r:0 w:1)
	/// Proof: `AccountMigration::Multisigs` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AccountMigration::Accounts` (r:1 w:1)
	/// Proof: `AccountMigration::Accounts` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `AccountMigration::KtonAccounts` (r:1 w:1)
	/// Proof: `AccountMigration::KtonAccounts` (`max_values`: None, `max_size`: Some(82), added: 2557, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Asset` (r:1 w:1)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(166), added: 2641, mode: `MaxEncodedLen`)
	/// Storage: `AccountMigration::Identities` (r:1 w:1)
	/// Proof: `AccountMigration::Identities` (`max_values`: None, `max_size`: Some(9219), added: 11694, mode: `MaxEncodedLen`)
	/// Storage: `Identity::Registrars` (r:1 w:1)
	/// Proof: `Identity::Registrars` (`max_values`: Some(1), `max_size`: Some(901), added: 1396, mode: `MaxEncodedLen`)
	/// Storage: `AccountMigration::Ledgers` (r:1 w:1)
	/// Proof: `AccountMigration::Ledgers` (`max_values`: None, `max_size`: Some(1845), added: 4320, mode: `MaxEncodedLen`)
	/// Storage: `AccountMigration::Deposits` (r:1 w:1)
	/// Proof: `AccountMigration::Deposits` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:2 w:3)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(116), added: 2591, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Account` (r:1 w:2)
	/// Proof: `Assets::Account` (`max_values`: None, `max_size`: Some(114), added: 2589, mode: `MaxEncodedLen`)
	/// Storage: `DarwiniaStaking::Ledgers` (r:0 w:1)
	/// Proof: `DarwiniaStaking::Ledgers` (`max_values`: None, `max_size`: Some(1833), added: 4308, mode: `MaxEncodedLen`)
	/// Storage: `Identity::IdentityOf` (r:0 w:1)
	/// Proof: `Identity::IdentityOf` (`max_values`: None, `max_size`: Some(7526), added: 10001, mode: `MaxEncodedLen`)
	/// Storage: `Deposit::Deposits` (r:0 w:1)
	/// Proof: `Deposit::Deposits` (`max_values`: None, `max_size`: Some(26150), added: 28625, mode: `MaxEncodedLen`)
	/// The range of component `x` is `[0, 99]`.
	/// The range of component `y` is `[0, 99]`.
	/// The range of component `z` is `[0, 99]`.
	fn migrate_multisig(x: u32, _y: u32, z: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `2 + x * (165 ±0) + z * (165 ±0)`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(29_199_710, 0)
			.saturating_add(Weight::from_parts(0, 2))
			// Standard Error: 17_482
			.saturating_add(Weight::from_parts(5_805, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(Weight::from_parts(0, 165).saturating_mul(x.into()))
			.saturating_add(Weight::from_parts(0, 165).saturating_mul(z.into()))
	}
	/// Storage: `AccountMigration::Multisigs` (r:1 w:1)
	/// Proof: `AccountMigration::Multisigs` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:0)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(116), added: 2591, mode: `MaxEncodedLen`)
	fn complete_multisig_migration() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `3499`
		//  Estimated: `6964`
		// Minimum execution time: 12_000_000 picoseconds.
		Weight::from_parts(12_000_000, 0)
			.saturating_add(Weight::from_parts(0, 6964))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
