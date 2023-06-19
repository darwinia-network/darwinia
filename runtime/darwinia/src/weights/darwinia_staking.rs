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

//! Autogenerated weights for `darwinia_staking`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-06-19, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `inv.cafe`, CPU: `13th Gen Intel(R) Core(TM) i9-13900K`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("darwinia-local"), DB CACHE: 1024

// Executed Command:
// target/release/darwinia
// benchmark
// pallet
// --header
// .maintain/license-header
// --execution
// wasm
// --heap-pages
// 4096
// --chain
// darwinia-local
// --output
// runtime/darwinia/src/weights
// --extrinsic
// *
// --pallet
// *
// --steps
// 50
// --repeat
// 20

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `darwinia_staking`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> darwinia_staking::WeightInfo for WeightInfo<T> {
	/// Storage: DarwiniaStaking Ledgers (r:1 w:1)
	/// Proof: DarwiniaStaking Ledgers (max_values: None, max_size: Some(1833), added: 4308, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	/// Storage: DarwiniaStaking RingPool (r:1 w:1)
	/// Proof: DarwiniaStaking RingPool (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:2 w:2)
	/// Proof: Assets Account (max_values: None, max_size: Some(94), added: 2569, mode: MaxEncodedLen)
	/// Storage: DarwiniaStaking KtonPool (r:1 w:1)
	/// Proof: DarwiniaStaking KtonPool (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
	/// Storage: Deposit Deposits (r:1 w:1)
	/// Proof: Deposit Deposits (max_values: None, max_size: Some(26150), added: 28625, mode: MaxEncodedLen)
	/// The range of component `x` is `[0, 1023]`.
	fn stake(x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1779`
		//  Estimated: `51255`
		// Minimum execution time: 52_657_000 picoseconds.
		Weight::from_parts(127_001_332, 0)
			.saturating_add(Weight::from_parts(0, 51255))
			// Standard Error: 1_871
			.saturating_add(Weight::from_parts(7_469, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	/// Storage: DarwiniaStaking Ledgers (r:1 w:1)
	/// Proof: DarwiniaStaking Ledgers (max_values: None, max_size: Some(1833), added: 4308, mode: MaxEncodedLen)
	/// Storage: DarwiniaStaking RingPool (r:1 w:1)
	/// Proof: DarwiniaStaking RingPool (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
	/// Storage: DarwiniaStaking KtonPool (r:1 w:1)
	/// Proof: DarwiniaStaking KtonPool (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
	/// Storage: Deposit Deposits (r:1 w:0)
	/// Proof: Deposit Deposits (max_values: None, max_size: Some(26150), added: 28625, mode: MaxEncodedLen)
	/// The range of component `x` is `[0, 1023]`.
	fn unstake(x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1463`
		//  Estimated: `37915`
		// Minimum execution time: 11_482_000 picoseconds.
		Weight::from_parts(54_442_906, 0)
			.saturating_add(Weight::from_parts(0, 37915))
			// Standard Error: 1_730
			.saturating_add(Weight::from_parts(9_447, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: DarwiniaStaking Ledgers (r:1 w:1)
	/// Proof: DarwiniaStaking Ledgers (max_values: None, max_size: Some(1833), added: 4308, mode: MaxEncodedLen)
	/// Storage: DarwiniaStaking RingPool (r:1 w:1)
	/// Proof: DarwiniaStaking RingPool (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
	/// Storage: DarwiniaStaking KtonPool (r:1 w:1)
	/// Proof: DarwiniaStaking KtonPool (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
	/// Storage: Deposit Deposits (r:1 w:0)
	/// Proof: Deposit Deposits (max_values: None, max_size: Some(26150), added: 28625, mode: MaxEncodedLen)
	/// The range of component `x` is `[0, 1023]`.
	fn restake(x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1567`
		//  Estimated: `37915`
		// Minimum execution time: 10_204_000 picoseconds.
		Weight::from_parts(46_352_901, 0)
			.saturating_add(Weight::from_parts(0, 37915))
			// Standard Error: 804
			.saturating_add(Weight::from_parts(4_432, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: DarwiniaStaking Ledgers (r:1 w:1)
	/// Proof: DarwiniaStaking Ledgers (max_values: None, max_size: Some(1833), added: 4308, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:2 w:2)
	/// Proof: Assets Account (max_values: None, max_size: Some(94), added: 2569, mode: MaxEncodedLen)
	/// Storage: Deposit Deposits (r:1 w:1)
	/// Proof: Deposit Deposits (max_values: None, max_size: Some(26150), added: 28625, mode: MaxEncodedLen)
	fn claim() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2089`
		//  Estimated: `48253`
		// Minimum execution time: 91_964_000 picoseconds.
		Weight::from_parts(95_743_000, 0)
			.saturating_add(Weight::from_parts(0, 48253))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	/// Storage: DarwiniaStaking Collators (r:1 w:1)
	/// Proof: DarwiniaStaking Collators (max_values: None, max_size: Some(32), added: 2507, mode: MaxEncodedLen)
	fn collect() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `392`
		//  Estimated: `3497`
		// Minimum execution time: 5_749_000 picoseconds.
		Weight::from_parts(6_015_000, 0)
			.saturating_add(Weight::from_parts(0, 3497))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: DarwiniaStaking Ledgers (r:1 w:0)
	/// Proof: DarwiniaStaking Ledgers (max_values: None, max_size: Some(1833), added: 4308, mode: MaxEncodedLen)
	/// Storage: DarwiniaStaking Collators (r:1 w:0)
	/// Proof: DarwiniaStaking Collators (max_values: None, max_size: Some(32), added: 2507, mode: MaxEncodedLen)
	/// Storage: DarwiniaStaking Nominators (r:1 w:1)
	/// Proof: DarwiniaStaking Nominators (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	fn nominate() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `697`
		//  Estimated: `12308`
		// Minimum execution time: 10_390_000 picoseconds.
		Weight::from_parts(11_083_000, 0)
			.saturating_add(Weight::from_parts(0, 12308))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: DarwiniaStaking Nominators (r:0 w:1)
	/// Proof: DarwiniaStaking Nominators (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	/// Storage: DarwiniaStaking Collators (r:0 w:1)
	/// Proof: DarwiniaStaking Collators (max_values: None, max_size: Some(32), added: 2507, mode: MaxEncodedLen)
	fn chill() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 3_624_000 picoseconds.
		Weight::from_parts(3_840_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: DarwiniaStaking CollatorCount (r:0 w:1)
	/// Proof: DarwiniaStaking CollatorCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	fn set_collator_count() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 2_009_000 picoseconds.
		Weight::from_parts(2_175_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
