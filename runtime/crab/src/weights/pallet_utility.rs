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

//! Autogenerated weights for `pallet_utility`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 47.2.0
//! DATE: 2025-06-02, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// frame-omni-bencher
// v1
// benchmark
// pallet
// --runtime
// target/release/wbuild/crab-runtime/crab_runtime.compact.compressed.wasm
// --genesis-builder
// none
// --pallet
// *
// --extrinsic
// *
// --header
// .maintain/license-header
// --template
// .maintain/pallet-weight-template.hbs
// --output
// runtime/crab/src/weights

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weights for `pallet_utility`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_utility::WeightInfo for WeightInfo<T> {
	/// Storage: `TxPause::PausedCalls` (r:1 w:0)
	/// Proof: `TxPause::PausedCalls` (`max_values`: None, `max_size`: Some(532), added: 3007, mode: `MaxEncodedLen`)
	/// The range of component `c` is `[0, 1000]`.
	fn batch(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `3997`
		// Minimum execution time: 3_000_000 picoseconds.
		Weight::from_parts(7_438_989, 3997)
			// Standard Error: 4_461
			.saturating_add(Weight::from_parts(4_137_217, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	/// Storage: `TxPause::PausedCalls` (r:1 w:0)
	/// Proof: `TxPause::PausedCalls` (`max_values`: None, `max_size`: Some(532), added: 3007, mode: `MaxEncodedLen`)
	fn as_derivative() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `3997`
		// Minimum execution time: 4_000_000 picoseconds.
		Weight::from_parts(5_000_000, 3997)
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	/// Storage: `TxPause::PausedCalls` (r:1 w:0)
	/// Proof: `TxPause::PausedCalls` (`max_values`: None, `max_size`: Some(532), added: 3007, mode: `MaxEncodedLen`)
	/// The range of component `c` is `[0, 1000]`.
	fn batch_all(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `3997`
		// Minimum execution time: 3_000_000 picoseconds.
		Weight::from_parts(13_981_948, 3997)
			// Standard Error: 4_302
			.saturating_add(Weight::from_parts(4_302_382, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	fn dispatch_as() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_000_000 picoseconds.
		Weight::from_parts(5_000_000, 0)
	}
	/// Storage: `TxPause::PausedCalls` (r:1 w:0)
	/// Proof: `TxPause::PausedCalls` (`max_values`: None, `max_size`: Some(532), added: 3007, mode: `MaxEncodedLen`)
	/// The range of component `c` is `[0, 1000]`.
	fn force_batch(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `3997`
		// Minimum execution time: 3_000_000 picoseconds.
		Weight::from_parts(3_000_000, 3997)
			// Standard Error: 2_742
			.saturating_add(Weight::from_parts(4_167_181, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
}
