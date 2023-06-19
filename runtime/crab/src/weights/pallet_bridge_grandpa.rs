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

//! Autogenerated weights for `pallet_bridge_grandpa`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-06-19, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `inv.cafe`, CPU: `13th Gen Intel(R) Core(TM) i9-13900K`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("crab-local"), DB CACHE: 1024

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
// crab-local
// --output
// runtime/crab/src/weights
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

/// Weight functions for `pallet_bridge_grandpa`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_bridge_grandpa::WeightInfo for WeightInfo<T> {
	/// Storage: BridgePolkadotGrandpa PalletOperatingMode (r:1 w:0)
	/// Proof: BridgePolkadotGrandpa PalletOperatingMode (max_values: Some(1), max_size: Some(1), added: 496, mode: MaxEncodedLen)
	/// Storage: BridgePolkadotGrandpa RequestCount (r:1 w:1)
	/// Proof: BridgePolkadotGrandpa RequestCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: BridgePolkadotGrandpa BestFinalized (r:1 w:1)
	/// Proof: BridgePolkadotGrandpa BestFinalized (max_values: Some(1), max_size: Some(36), added: 531, mode: MaxEncodedLen)
	/// Storage: BridgePolkadotGrandpa ImportedHeaders (r:1 w:2)
	/// Proof: BridgePolkadotGrandpa ImportedHeaders (max_values: None, max_size: Some(65568), added: 68043, mode: MaxEncodedLen)
	/// Storage: BridgePolkadotGrandpa CurrentAuthoritySet (r:1 w:0)
	/// Proof: BridgePolkadotGrandpa CurrentAuthoritySet (max_values: Some(1), max_size: Some(163850), added: 164345, mode: MaxEncodedLen)
	/// Storage: BridgePolkadotGrandpa ImportedHashesPointer (r:1 w:1)
	/// Proof: BridgePolkadotGrandpa ImportedHashesPointer (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: BridgePolkadotGrandpa ImportedHashes (r:1 w:1)
	/// Proof: BridgePolkadotGrandpa ImportedHashes (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
	/// The range of component `p` is `[51, 102]`.
	/// The range of component `v` is `[50, 100]`.
	fn submit_finality_proof(p: u32, _v: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2544 + p * (40 ±0)`
		//  Estimated: `243854`
		// Minimum execution time: 1_584_217_000 picoseconds.
		Weight::from_parts(1_617_253_000, 0)
			.saturating_add(Weight::from_parts(0, 243854))
			// Standard Error: 160_970
			.saturating_add(Weight::from_parts(20_351_393, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(6))
	}
}
