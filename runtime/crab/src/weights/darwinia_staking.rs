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

//! Autogenerated weights for `darwinia_staking`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-03-06, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("crab-dev")`, DB CACHE: 1024

// Executed Command:
// target/release/darwinia
// benchmark
// pallet
// --header
// .maintain/license-header
// --heap-pages
// 4096
// --chain
// crab-dev
// --output
// runtime/crab/src/weights
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

/// Weight functions for `darwinia_staking`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> darwinia_staking::WeightInfo for WeightInfo<T> {
	/// Storage: `DarwiniaStaking::Ledgers` (r:1 w:1)
	/// Proof: `DarwiniaStaking::Ledgers` (`max_values`: None, `max_size`: Some(1833), added: 4308, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(116), added: 2591, mode: `MaxEncodedLen`)
	/// Storage: `DarwiniaStaking::RingPool` (r:1 w:1)
	/// Proof: `DarwiniaStaking::RingPool` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Asset` (r:1 w:1)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(166), added: 2641, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Account` (r:2 w:2)
	/// Proof: `Assets::Account` (`max_values`: None, `max_size`: Some(114), added: 2589, mode: `MaxEncodedLen`)
	/// Storage: `DarwiniaStaking::KtonPool` (r:1 w:1)
	/// Proof: `DarwiniaStaking::KtonPool` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `Deposit::Deposits` (r:1 w:1)
	/// Proof: `Deposit::Deposits` (`max_values`: None, `max_size`: Some(26150), added: 28625, mode: `MaxEncodedLen`)
	/// The range of component `x` is `[0, 1023]`.
	fn stake(x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1877`
		//  Estimated: `29615`
		// Minimum execution time: 75_000_000 picoseconds.
		Weight::from_parts(150_556_965, 0)
			.saturating_add(Weight::from_parts(0, 29615))
			// Standard Error: 1_694
			.saturating_add(Weight::from_parts(7_352, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	/// Storage: `DarwiniaStaking::Ledgers` (r:1 w:1)
	/// Proof: `DarwiniaStaking::Ledgers` (`max_values`: None, `max_size`: Some(1833), added: 4308, mode: `MaxEncodedLen`)
	/// Storage: `DarwiniaStaking::RingPool` (r:1 w:1)
	/// Proof: `DarwiniaStaking::RingPool` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `DarwiniaStaking::KtonPool` (r:1 w:1)
	/// Proof: `DarwiniaStaking::KtonPool` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Asset` (r:1 w:1)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(166), added: 2641, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Account` (r:2 w:2)
	/// Proof: `Assets::Account` (`max_values`: None, `max_size`: Some(114), added: 2589, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(116), added: 2591, mode: `MaxEncodedLen`)
	/// Storage: `Deposit::Deposits` (r:1 w:0)
	/// Proof: `Deposit::Deposits` (`max_values`: None, `max_size`: Some(26150), added: 28625, mode: `MaxEncodedLen`)
	/// The range of component `x` is `[0, 1023]`.
	fn unstake(x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2054`
		//  Estimated: `29615`
		// Minimum execution time: 42_000_000 picoseconds.
		Weight::from_parts(95_224_175, 0)
			.saturating_add(Weight::from_parts(0, 29615))
			// Standard Error: 1_151
			.saturating_add(Weight::from_parts(6_978, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	/// Storage: `DarwiniaStaking::Ledgers` (r:1 w:1)
	/// Proof: `DarwiniaStaking::Ledgers` (`max_values`: None, `max_size`: Some(1833), added: 4308, mode: `MaxEncodedLen`)
	/// Storage: `DarwiniaStaking::RingPool` (r:1 w:1)
	/// Proof: `DarwiniaStaking::RingPool` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `Deposit::Deposits` (r:1 w:0)
	/// Proof: `Deposit::Deposits` (`max_values`: None, `max_size`: Some(26150), added: 28625, mode: `MaxEncodedLen`)
	/// The range of component `x` is `[0, 1023]`.
	fn restake(x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1570`
		//  Estimated: `29615`
		// Minimum execution time: 9_000_000 picoseconds.
		Weight::from_parts(53_542_194, 0)
			.saturating_add(Weight::from_parts(0, 29615))
			// Standard Error: 966
			.saturating_add(Weight::from_parts(4_864, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `DarwiniaStaking::Ledgers` (r:1 w:1)
	/// Proof: `DarwiniaStaking::Ledgers` (`max_values`: None, `max_size`: Some(1833), added: 4308, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(116), added: 2591, mode: `MaxEncodedLen`)
	/// Storage: `Deposit::Deposits` (r:1 w:1)
	/// Proof: `Deposit::Deposits` (`max_values`: None, `max_size`: Some(26150), added: 28625, mode: `MaxEncodedLen`)
	fn claim() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1658`
		//  Estimated: `29615`
		// Minimum execution time: 81_000_000 picoseconds.
		Weight::from_parts(83_000_000, 0)
			.saturating_add(Weight::from_parts(0, 29615))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `DarwiniaStaking::Collators` (r:1 w:1)
	/// Proof: `DarwiniaStaking::Collators` (`max_values`: None, `max_size`: Some(32), added: 2507, mode: `MaxEncodedLen`)
	fn collect() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `377`
		//  Estimated: `3497`
		// Minimum execution time: 10_000_000 picoseconds.
		Weight::from_parts(11_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3497))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `DarwiniaStaking::Ledgers` (r:1 w:0)
	/// Proof: `DarwiniaStaking::Ledgers` (`max_values`: None, `max_size`: Some(1833), added: 4308, mode: `MaxEncodedLen`)
	/// Storage: `DarwiniaStaking::Collators` (r:1 w:0)
	/// Proof: `DarwiniaStaking::Collators` (`max_values`: None, `max_size`: Some(32), added: 2507, mode: `MaxEncodedLen`)
	/// Storage: `DarwiniaStaking::Nominators` (r:1 w:1)
	/// Proof: `DarwiniaStaking::Nominators` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	fn nominate() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `545`
		//  Estimated: `5298`
		// Minimum execution time: 11_000_000 picoseconds.
		Weight::from_parts(11_000_000, 0)
			.saturating_add(Weight::from_parts(0, 5298))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `DarwiniaStaking::Nominators` (r:0 w:1)
	/// Proof: `DarwiniaStaking::Nominators` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `DarwiniaStaking::Collators` (r:0 w:1)
	/// Proof: `DarwiniaStaking::Collators` (`max_values`: None, `max_size`: Some(32), added: 2507, mode: `MaxEncodedLen`)
	fn chill() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 3_000_000 picoseconds.
		Weight::from_parts(4_000_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `DarwiniaStaking::ExposureCacheStates` (r:1 w:0)
	/// Proof: `DarwiniaStaking::ExposureCacheStates` (`max_values`: Some(1), `max_size`: Some(3), added: 498, mode: `MaxEncodedLen`)
	/// Storage: `DarwiniaStaking::ExposureCache2` (r:1 w:1)
	/// Proof: `DarwiniaStaking::ExposureCache2` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `DarwiniaStaking::PendingRewards` (r:1 w:1)
	/// Proof: `DarwiniaStaking::PendingRewards` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:0)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(116), added: 2591, mode: `MaxEncodedLen`)
	fn payout() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1382`
		//  Estimated: `4847`
		// Minimum execution time: 174_000_000 picoseconds.
		Weight::from_parts(176_000_000, 0)
			.saturating_add(Weight::from_parts(0, 4847))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `DarwiniaStaking::CollatorCount` (r:0 w:1)
	/// Proof: `DarwiniaStaking::CollatorCount` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	fn set_collator_count() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 2_000_000 picoseconds.
		Weight::from_parts(2_000_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
