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

//! Autogenerated weights for `pallet_bridge_parachains`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-05-24, STEPS: `2`, REPEAT: `1`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("darwinia-local"), DB CACHE: 1024

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
// --chain
// darwinia-local
// --output
// runtime/darwinia/src/weights/
// --extrinsic
// *
// --pallet
// pallet_bridge_parachains

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_bridge_parachains`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_bridge_parachains::WeightInfo for WeightInfo<T> {
	/// Storage: BridgeKusamaParachain PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeKusamaParachain PalletOperatingMode (max_values: Some(1), max_size: Some(1), added: 496, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgeKusamaGrandpa ImportedHeaders (max_values: None, max_size: Some(131104), added: 133579, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaParachain ParasInfo (r:1024 w:1024)
	/// Proof: BridgeKusamaParachain ParasInfo (max_values: None, max_size: Some(60), added: 2535, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaParachain ImportedParaHashes (r:1024 w:1024)
	/// Proof: BridgeKusamaParachain ImportedParaHashes (max_values: None, max_size: Some(64), added: 2539, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaParachain ImportedParaHeads (r:0 w:1024)
	/// Proof: BridgeKusamaParachain ImportedParaHeads (max_values: None, max_size: Some(1092), added: 3567, mode: MaxEncodedLen)
	/// The range of component `p` is `[1, 1024]`.
	fn submit_parachain_heads_with_n_parachains(_p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `451`
		//  Estimated: `5333811`
		// Minimum execution time: 62_997_000 picoseconds.
		Weight::from_parts(36_514_055_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().reads(2050))
			.saturating_add(T::DbWeight::get().writes(3072))
	}
	/// Storage: BridgeKusamaParachain PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeKusamaParachain PalletOperatingMode (max_values: Some(1), max_size: Some(1), added: 496, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgeKusamaGrandpa ImportedHeaders (max_values: None, max_size: Some(131104), added: 133579, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaParachain ParasInfo (r:1 w:1)
	/// Proof: BridgeKusamaParachain ParasInfo (max_values: None, max_size: Some(60), added: 2535, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaParachain ImportedParaHashes (r:1 w:1)
	/// Proof: BridgeKusamaParachain ImportedParaHashes (max_values: None, max_size: Some(64), added: 2539, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaParachain ImportedParaHeads (r:0 w:1)
	/// Proof: BridgeKusamaParachain ImportedParaHeads (max_values: None, max_size: Some(1092), added: 3567, mode: MaxEncodedLen)
	fn submit_parachain_heads_with_1kb_proof() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `451`
		//  Estimated: `143109`
		// Minimum execution time: 96_381_000 picoseconds.
		Weight::from_parts(96_381_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: BridgeKusamaParachain PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeKusamaParachain PalletOperatingMode (max_values: Some(1), max_size: Some(1), added: 496, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgeKusamaGrandpa ImportedHeaders (max_values: None, max_size: Some(131104), added: 133579, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaParachain ParasInfo (r:1 w:1)
	/// Proof: BridgeKusamaParachain ParasInfo (max_values: None, max_size: Some(60), added: 2535, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaParachain ImportedParaHashes (r:1 w:1)
	/// Proof: BridgeKusamaParachain ImportedParaHashes (max_values: None, max_size: Some(64), added: 2539, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaParachain ImportedParaHeads (r:0 w:1)
	/// Proof: BridgeKusamaParachain ImportedParaHeads (max_values: None, max_size: Some(1092), added: 3567, mode: MaxEncodedLen)
	fn submit_parachain_heads_with_16kb_proof() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `451`
		//  Estimated: `143109`
		// Minimum execution time: 133_328_000 picoseconds.
		Weight::from_parts(133_328_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
}
