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

//! Autogenerated weights for `pallet_asset_manager`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 32.0.0
//! DATE: 2024-11-26, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
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

/// Weight functions for `pallet_asset_manager`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_asset_manager::WeightInfo for WeightInfo<T> {
	/// Storage: `AssetManager::AssetIdType` (r:1 w:1)
	/// Proof: `AssetManager::AssetIdType` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Assets::Asset` (r:1 w:1)
	/// Proof: `Assets::Asset` (`max_values`: None, `max_size`: Some(166), added: 2641, mode: `MaxEncodedLen`)
	/// Storage: `Assets::Metadata` (r:1 w:1)
	/// Proof: `Assets::Metadata` (`max_values`: None, `max_size`: Some(144), added: 2619, mode: `MaxEncodedLen`)
	/// Storage: `AssetManager::AssetTypeId` (r:0 w:1)
	/// Proof: `AssetManager::AssetTypeId` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn register_foreign_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `276`
		//  Estimated: `3741`
		// Minimum execution time: 26_000_000 picoseconds.
		Weight::from_parts(27_000_000, 0)
			.saturating_add(Weight::from_parts(0, 3741))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: `AssetManager::SupportedFeePaymentAssets` (r:1 w:1)
	/// Proof: `AssetManager::SupportedFeePaymentAssets` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `AssetManager::AssetIdType` (r:1 w:1)
	/// Proof: `AssetManager::AssetIdType` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AssetManager::AssetTypeUnitsPerSecond` (r:1 w:2)
	/// Proof: `AssetManager::AssetTypeUnitsPerSecond` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AssetManager::AssetTypeId` (r:0 w:2)
	/// Proof: `AssetManager::AssetTypeId` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `x` is `[5, 100]`.
	fn change_existing_asset_type() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `968 + x * (13 ±0)`
		//  Estimated: `4303 + x * (14 ±0)`
		// Minimum execution time: 20_000_000 picoseconds.
		Weight::from_parts(23_199_447, 0)
			.saturating_add(Weight::from_parts(0, 4303))
			// Standard Error: 2_109
			.saturating_add(Weight::from_parts(301_010, 0))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(6))
			.saturating_add(Weight::from_parts(0, 14))
	}
	/// Storage: `AssetManager::AssetIdType` (r:1 w:1)
	/// Proof: `AssetManager::AssetIdType` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `AssetManager::AssetTypeId` (r:0 w:1)
	/// Proof: `AssetManager::AssetTypeId` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `x` is `[5, 100]`.
	fn remove_existing_asset_type() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `447 + x * (4 ±0)`
		//  Estimated: `3909 + x * (5 ±0)`
		// Minimum execution time: 14_047_000 picoseconds.
		Weight::from_parts(15_828_607, 3909)
			// Standard Error: 1_451
			.saturating_add(Weight::from_parts(74_700, 0))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
			.saturating_add(Weight::from_parts(0, 5))
	}
}
