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

//! Autogenerated weights for `pallet_assets`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-06-19, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `inv.cafe`, CPU: `13th Gen Intel(R) Core(TM) i9-13900K`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("pangoro-local"), DB CACHE: 1024

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
// pangoro-local
// --output
// runtime/pangoro/src/weights
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

/// Weight functions for `pallet_assets`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_assets::WeightInfo for WeightInfo<T> {
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	fn create() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `120`
		//  Estimated: `3631`
		// Minimum execution time: 9_383_000 picoseconds.
		Weight::from_parts(9_898_000, 0)
			.saturating_add(Weight::from_parts(0, 3631))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	fn force_create() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `120`
		//  Estimated: `3631`
		// Minimum execution time: 9_052_000 picoseconds.
		Weight::from_parts(9_444_000, 0)
			.saturating_add(Weight::from_parts(0, 3631))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	fn start_destroy() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `270`
		//  Estimated: `3631`
		// Minimum execution time: 9_047_000 picoseconds.
		Weight::from_parts(9_545_000, 0)
			.saturating_add(Weight::from_parts(0, 3631))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:1001 w:1000)
	/// Proof: Assets Account (max_values: None, max_size: Some(94), added: 2569, mode: MaxEncodedLen)
	/// Storage: System Account (r:1000 w:1000)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	/// The range of component `c` is `[0, 1000]`.
	fn destroy_accounts(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `436 + c * (183 ±0)`
		//  Estimated: `8180 + c * (5160 ±0)`
		// Minimum execution time: 12_110_000 picoseconds.
		Weight::from_parts(12_396_000, 0)
			.saturating_add(Weight::from_parts(0, 8180))
			// Standard Error: 7_728
			.saturating_add(Weight::from_parts(9_038_667, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(c.into())))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(c.into())))
			.saturating_add(Weight::from_parts(0, 5160).saturating_mul(c.into()))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Approvals (r:1001 w:1000)
	/// Proof: Assets Approvals (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// The range of component `a` is `[0, 1000]`.
	fn destroy_approvals(a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `421 + a * (74 ±0)`
		//  Estimated: `7224 + a * (2603 ±0)`
		// Minimum execution time: 11_898_000 picoseconds.
		Weight::from_parts(12_215_000, 0)
			.saturating_add(Weight::from_parts(0, 7224))
			// Standard Error: 3_173
			.saturating_add(Weight::from_parts(3_241_749, 0).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(a.into())))
			.saturating_add(Weight::from_parts(0, 2603).saturating_mul(a.into()))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Metadata (r:1 w:0)
	/// Proof: Assets Metadata (max_values: None, max_size: Some(144), added: 2619, mode: MaxEncodedLen)
	fn finish_destroy() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `313`
		//  Estimated: `7240`
		// Minimum execution time: 10_368_000 picoseconds.
		Weight::from_parts(10_733_000, 0)
			.saturating_add(Weight::from_parts(0, 7240))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:1 w:1)
	/// Proof: Assets Account (max_values: None, max_size: Some(94), added: 2569, mode: MaxEncodedLen)
	fn mint() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `313`
		//  Estimated: `7190`
		// Minimum execution time: 19_324_000 picoseconds.
		Weight::from_parts(20_064_000, 0)
			.saturating_add(Weight::from_parts(0, 7190))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:1 w:1)
	/// Proof: Assets Account (max_values: None, max_size: Some(94), added: 2569, mode: MaxEncodedLen)
	fn burn() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `373`
		//  Estimated: `7190`
		// Minimum execution time: 22_932_000 picoseconds.
		Weight::from_parts(23_439_000, 0)
			.saturating_add(Weight::from_parts(0, 7190))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:2 w:2)
	/// Proof: Assets Account (max_values: None, max_size: Some(94), added: 2569, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	fn transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `464`
		//  Estimated: `13340`
		// Minimum execution time: 32_898_000 picoseconds.
		Weight::from_parts(34_019_000, 0)
			.saturating_add(Weight::from_parts(0, 13340))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:2 w:2)
	/// Proof: Assets Account (max_values: None, max_size: Some(94), added: 2569, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	fn transfer_keep_alive() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `464`
		//  Estimated: `13340`
		// Minimum execution time: 29_657_000 picoseconds.
		Weight::from_parts(32_496_000, 0)
			.saturating_add(Weight::from_parts(0, 13340))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:2 w:2)
	/// Proof: Assets Account (max_values: None, max_size: Some(94), added: 2569, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	fn force_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `464`
		//  Estimated: `13340`
		// Minimum execution time: 33_118_000 picoseconds.
		Weight::from_parts(34_211_000, 0)
			.saturating_add(Weight::from_parts(0, 13340))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: Assets Asset (r:1 w:0)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:1 w:1)
	/// Proof: Assets Account (max_values: None, max_size: Some(94), added: 2569, mode: MaxEncodedLen)
	fn freeze() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `373`
		//  Estimated: `7190`
		// Minimum execution time: 11_797_000 picoseconds.
		Weight::from_parts(12_483_000, 0)
			.saturating_add(Weight::from_parts(0, 7190))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:0)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:1 w:1)
	/// Proof: Assets Account (max_values: None, max_size: Some(94), added: 2569, mode: MaxEncodedLen)
	fn thaw() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `373`
		//  Estimated: `7190`
		// Minimum execution time: 11_643_000 picoseconds.
		Weight::from_parts(12_527_000, 0)
			.saturating_add(Weight::from_parts(0, 7190))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	fn freeze_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `270`
		//  Estimated: `3631`
		// Minimum execution time: 8_808_000 picoseconds.
		Weight::from_parts(9_346_000, 0)
			.saturating_add(Weight::from_parts(0, 3631))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	fn thaw_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `270`
		//  Estimated: `3631`
		// Minimum execution time: 8_823_000 picoseconds.
		Weight::from_parts(9_459_000, 0)
			.saturating_add(Weight::from_parts(0, 3631))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Metadata (r:1 w:0)
	/// Proof: Assets Metadata (max_values: None, max_size: Some(144), added: 2619, mode: MaxEncodedLen)
	fn transfer_ownership() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `313`
		//  Estimated: `7240`
		// Minimum execution time: 10_990_000 picoseconds.
		Weight::from_parts(11_530_000, 0)
			.saturating_add(Weight::from_parts(0, 7240))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	fn set_team() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `270`
		//  Estimated: `3631`
		// Minimum execution time: 9_603_000 picoseconds.
		Weight::from_parts(10_223_000, 0)
			.saturating_add(Weight::from_parts(0, 3631))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:0)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Metadata (r:1 w:1)
	/// Proof: Assets Metadata (max_values: None, max_size: Some(144), added: 2619, mode: MaxEncodedLen)
	/// The range of component `n` is `[0, 50]`.
	/// The range of component `s` is `[0, 50]`.
	fn set_metadata(_n: u32, _s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `313`
		//  Estimated: `7240`
		// Minimum execution time: 11_029_000 picoseconds.
		Weight::from_parts(12_818_841, 0)
			.saturating_add(Weight::from_parts(0, 7240))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:0)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Metadata (r:1 w:1)
	/// Proof: Assets Metadata (max_values: None, max_size: Some(144), added: 2619, mode: MaxEncodedLen)
	fn clear_metadata() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `441`
		//  Estimated: `7240`
		// Minimum execution time: 11_104_000 picoseconds.
		Weight::from_parts(11_504_000, 0)
			.saturating_add(Weight::from_parts(0, 7240))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:0)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Metadata (r:1 w:1)
	/// Proof: Assets Metadata (max_values: None, max_size: Some(144), added: 2619, mode: MaxEncodedLen)
	/// The range of component `n` is `[0, 50]`.
	/// The range of component `s` is `[0, 50]`.
	fn force_set_metadata(n: u32, _s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `200`
		//  Estimated: `7240`
		// Minimum execution time: 10_493_000 picoseconds.
		Weight::from_parts(12_196_425, 0)
			.saturating_add(Weight::from_parts(0, 7240))
			// Standard Error: 6_056
			.saturating_add(Weight::from_parts(14_271, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:0)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Metadata (r:1 w:1)
	/// Proof: Assets Metadata (max_values: None, max_size: Some(144), added: 2619, mode: MaxEncodedLen)
	fn force_clear_metadata() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `441`
		//  Estimated: `7240`
		// Minimum execution time: 10_864_000 picoseconds.
		Weight::from_parts(11_228_000, 0)
			.saturating_add(Weight::from_parts(0, 7240))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	fn force_asset_status() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `270`
		//  Estimated: `3631`
		// Minimum execution time: 8_710_000 picoseconds.
		Weight::from_parts(9_279_000, 0)
			.saturating_add(Weight::from_parts(0, 3631))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Approvals (r:1 w:1)
	/// Proof: Assets Approvals (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn approve_transfer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `270`
		//  Estimated: `7224`
		// Minimum execution time: 13_834_000 picoseconds.
		Weight::from_parts(14_712_000, 0)
			.saturating_add(Weight::from_parts(0, 7224))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Approvals (r:1 w:1)
	/// Proof: Assets Approvals (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:2 w:2)
	/// Proof: Assets Account (max_values: None, max_size: Some(94), added: 2569, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	fn transfer_approved() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `562`
		//  Estimated: `16933`
		// Minimum execution time: 38_255_000 picoseconds.
		Weight::from_parts(39_249_000, 0)
			.saturating_add(Weight::from_parts(0, 16933))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Approvals (r:1 w:1)
	/// Proof: Assets Approvals (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn cancel_approval() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `420`
		//  Estimated: `7224`
		// Minimum execution time: 14_767_000 picoseconds.
		Weight::from_parts(15_274_000, 0)
			.saturating_add(Weight::from_parts(0, 7224))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	/// Storage: Assets Approvals (r:1 w:1)
	/// Proof: Assets Approvals (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn force_cancel_approval() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `420`
		//  Estimated: `7224`
		// Minimum execution time: 14_801_000 picoseconds.
		Weight::from_parts(15_458_000, 0)
			.saturating_add(Weight::from_parts(0, 7224))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(166), added: 2641, mode: MaxEncodedLen)
	fn set_min_balance() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `270`
		//  Estimated: `3631`
		// Minimum execution time: 10_267_000 picoseconds.
		Weight::from_parts(10_823_000, 0)
			.saturating_add(Weight::from_parts(0, 3631))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Assets Account (r:1 w:1)
	/// Proof: Assets Account (max_values: None, max_size: Some(134), added: 2609, mode: MaxEncodedLen)
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn touch() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `453`
		//  Estimated: `3675`
		// Minimum execution time: 37_468_000 picoseconds.
		Weight::from_parts(37_957_000, 3675)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: Assets Account (r:1 w:1)
	/// Proof: Assets Account (max_values: None, max_size: Some(134), added: 2609, mode: MaxEncodedLen)
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	fn touch_other() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `351`
		//  Estimated: `3675`
		// Minimum execution time: 383_408_000 picoseconds.
		Weight::from_parts(392_036_000, 3675)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Assets Account (r:1 w:1)
	/// Proof: Assets Account (max_values: None, max_size: Some(134), added: 2609, mode: MaxEncodedLen)
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn refund() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `579`
		//  Estimated: `3675`
		// Minimum execution time: 34_066_000 picoseconds.
		Weight::from_parts(34_347_000, 3675)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: Assets Account (r:1 w:1)
	/// Proof: Assets Account (max_values: None, max_size: Some(134), added: 2609, mode: MaxEncodedLen)
	/// Storage: Assets Asset (r:1 w:1)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	fn refund_other() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `510`
		//  Estimated: `3675`
		// Minimum execution time: 32_060_000 picoseconds.
		Weight::from_parts(32_519_000, 3675)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: Assets Asset (r:1 w:0)
	/// Proof: Assets Asset (max_values: None, max_size: Some(210), added: 2685, mode: MaxEncodedLen)
	/// Storage: Assets Account (r:1 w:1)
	/// Proof: Assets Account (max_values: None, max_size: Some(134), added: 2609, mode: MaxEncodedLen)
	fn block() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `459`
		//  Estimated: `3675`
		// Minimum execution time: 115_000_000 picoseconds.
		Weight::from_parts(163_000_000, 3675)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}
