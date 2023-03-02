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

//! Autogenerated weights for `pallet_utility`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-02-22, STEPS: `2`, REPEAT: 1, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `Debian`, CPU: `12th Gen Intel(R) Core(TM) i9-12900K`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("crab-local"), DB CACHE: 1024

// Executed Command:
// ./target/release/darwinia
// benchmark
// pallet
// --header
// .maintain/license-header
// --execution
// wasm
// --heap-pages
// 4096
// --steps
// 2
// --repeat
// 1
// --chain
// crab-local
// --output
// runtime/crab/src/weights/
// --extrinsic
// *
// --pallet
// *

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_utility`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_utility::WeightInfo for WeightInfo<T> {
	/// The range of component `c` is `[0, 1000]`.
	fn batch(_c: u32, ) -> Weight {
		// Minimum execution time: 38_797 nanoseconds.
		Weight::from_ref_time(3_794_342_000)
	}
	fn as_derivative() -> Weight {
		// Minimum execution time: 22_134 nanoseconds.
		Weight::from_ref_time(22_134_000)
	}
	/// The range of component `c` is `[0, 1000]`.
	fn batch_all(_c: u32, ) -> Weight {
		// Minimum execution time: 30_681 nanoseconds.
		Weight::from_ref_time(3_916_476_000)
	}
	fn dispatch_as() -> Weight {
		// Minimum execution time: 29_357 nanoseconds.
		Weight::from_ref_time(29_357_000)
	}
	/// The range of component `c` is `[0, 1000]`.
	fn force_batch(_c: u32, ) -> Weight {
		// Minimum execution time: 30_220 nanoseconds.
		Weight::from_ref_time(5_327_665_000)
	}
}