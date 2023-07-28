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

//! Autogenerated weights for `pallet_bridge_messages`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-07-28, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `inv.cafe`, CPU: `13th Gen Intel(R) Core(TM) i9-13900K`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("crab-dev"), DB CACHE: 1024

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
// crab-dev
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
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_bridge_messages`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_bridge_messages::WeightInfo for WeightInfo<T> {
	/// Storage: BridgeDarwiniaMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeDarwiniaMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgeDarwiniaMessages OutboundLanes (r:1 w:1)
	/// Proof: BridgeDarwiniaMessages OutboundLanes (max_values: None, max_size: Some(44), added: 2519, mode: MaxEncodedLen)
	/// Storage: System Account (r:2 w:2)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	/// Storage: DarwiniaFeeMarket AssignedRelayers (r:1 w:0)
	/// Proof Skipped: DarwiniaFeeMarket AssignedRelayers (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: BridgeDarwiniaMessages OutboundMessages (r:0 w:9)
	/// Proof: BridgeDarwiniaMessages OutboundMessages (max_values: None, max_size: Some(2621484), added: 2623959, mode: MaxEncodedLen)
	fn send_minimal_message_worst_case() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `439`
		//  Estimated: `6172`
		// Minimum execution time: 56_980_000 picoseconds.
		Weight::from_parts(59_023_000, 0)
			.saturating_add(Weight::from_parts(0, 6172))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(12))
	}
	/// Storage: BridgeDarwiniaMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeDarwiniaMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgeDarwiniaMessages OutboundLanes (r:1 w:1)
	/// Proof: BridgeDarwiniaMessages OutboundLanes (max_values: None, max_size: Some(44), added: 2519, mode: MaxEncodedLen)
	/// Storage: System Account (r:2 w:2)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	/// Storage: DarwiniaFeeMarket AssignedRelayers (r:1 w:0)
	/// Proof Skipped: DarwiniaFeeMarket AssignedRelayers (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: BridgeDarwiniaMessages OutboundMessages (r:0 w:9)
	/// Proof: BridgeDarwiniaMessages OutboundMessages (max_values: None, max_size: Some(2621484), added: 2623959, mode: MaxEncodedLen)
	fn send_1_kb_message_worst_case() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `439`
		//  Estimated: `6172`
		// Minimum execution time: 57_743_000 picoseconds.
		Weight::from_parts(58_978_000, 0)
			.saturating_add(Weight::from_parts(0, 6172))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(12))
	}
	/// Storage: BridgeDarwiniaMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeDarwiniaMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgeDarwiniaMessages OutboundLanes (r:1 w:1)
	/// Proof: BridgeDarwiniaMessages OutboundLanes (max_values: None, max_size: Some(44), added: 2519, mode: MaxEncodedLen)
	/// Storage: System Account (r:2 w:2)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	/// Storage: DarwiniaFeeMarket AssignedRelayers (r:1 w:0)
	/// Proof Skipped: DarwiniaFeeMarket AssignedRelayers (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: BridgeDarwiniaMessages OutboundMessages (r:0 w:9)
	/// Proof: BridgeDarwiniaMessages OutboundMessages (max_values: None, max_size: Some(2621484), added: 2623959, mode: MaxEncodedLen)
	fn send_16_kb_message_worst_case() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `439`
		//  Estimated: `6172`
		// Minimum execution time: 62_656_000 picoseconds.
		Weight::from_parts(64_352_000, 0)
			.saturating_add(Weight::from_parts(0, 6172))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(12))
	}
	/// Storage: BridgeDarwiniaMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeDarwiniaMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgePolkadotGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgePolkadotGrandpa ImportedHeaders (max_values: None, max_size: Some(65568), added: 68043, mode: MaxEncodedLen)
	/// Storage: BridgeDarwiniaMessages InboundLanes (r:1 w:1)
	/// Proof: BridgeDarwiniaMessages InboundLanes (max_values: None, max_size: Some(5660), added: 8135, mode: MaxEncodedLen)
	fn receive_single_message_proof() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `576`
		//  Estimated: `69033`
		// Minimum execution time: 28_366_000 picoseconds.
		Weight::from_parts(29_640_000, 0)
			.saturating_add(Weight::from_parts(0, 69033))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: BridgeDarwiniaMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeDarwiniaMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgePolkadotGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgePolkadotGrandpa ImportedHeaders (max_values: None, max_size: Some(65568), added: 68043, mode: MaxEncodedLen)
	/// Storage: BridgeDarwiniaMessages InboundLanes (r:1 w:1)
	/// Proof: BridgeDarwiniaMessages InboundLanes (max_values: None, max_size: Some(5660), added: 8135, mode: MaxEncodedLen)
	fn receive_two_messages_proof() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `576`
		//  Estimated: `69033`
		// Minimum execution time: 39_964_000 picoseconds.
		Weight::from_parts(40_425_000, 0)
			.saturating_add(Weight::from_parts(0, 69033))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: BridgeDarwiniaMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeDarwiniaMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgePolkadotGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgePolkadotGrandpa ImportedHeaders (max_values: None, max_size: Some(65568), added: 68043, mode: MaxEncodedLen)
	/// Storage: BridgeDarwiniaMessages InboundLanes (r:1 w:1)
	/// Proof: BridgeDarwiniaMessages InboundLanes (max_values: None, max_size: Some(5660), added: 8135, mode: MaxEncodedLen)
	fn receive_single_message_proof_with_outbound_lane_state() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `576`
		//  Estimated: `69033`
		// Minimum execution time: 32_245_000 picoseconds.
		Weight::from_parts(33_001_000, 0)
			.saturating_add(Weight::from_parts(0, 69033))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: BridgeDarwiniaMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeDarwiniaMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgePolkadotGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgePolkadotGrandpa ImportedHeaders (max_values: None, max_size: Some(65568), added: 68043, mode: MaxEncodedLen)
	/// Storage: BridgeDarwiniaMessages InboundLanes (r:1 w:1)
	/// Proof: BridgeDarwiniaMessages InboundLanes (max_values: None, max_size: Some(5660), added: 8135, mode: MaxEncodedLen)
	fn receive_single_message_proof_1_kb() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `576`
		//  Estimated: `69033`
		// Minimum execution time: 33_556_000 picoseconds.
		Weight::from_parts(34_376_000, 0)
			.saturating_add(Weight::from_parts(0, 69033))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: BridgeDarwiniaMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeDarwiniaMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgePolkadotGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgePolkadotGrandpa ImportedHeaders (max_values: None, max_size: Some(65568), added: 68043, mode: MaxEncodedLen)
	/// Storage: BridgeDarwiniaMessages InboundLanes (r:1 w:1)
	/// Proof: BridgeDarwiniaMessages InboundLanes (max_values: None, max_size: Some(5660), added: 8135, mode: MaxEncodedLen)
	fn receive_single_message_proof_16_kb() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `576`
		//  Estimated: `69033`
		// Minimum execution time: 67_810_000 picoseconds.
		Weight::from_parts(71_277_000, 0)
			.saturating_add(Weight::from_parts(0, 69033))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: BridgeDarwiniaMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeDarwiniaMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgePolkadotGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgePolkadotGrandpa ImportedHeaders (max_values: None, max_size: Some(65568), added: 68043, mode: MaxEncodedLen)
	/// Storage: BridgeDarwiniaMessages InboundLanes (r:1 w:1)
	/// Proof: BridgeDarwiniaMessages InboundLanes (max_values: None, max_size: Some(5660), added: 8135, mode: MaxEncodedLen)
	fn receive_single_prepaid_message_proof() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `576`
		//  Estimated: `69033`
		// Minimum execution time: 28_418_000 picoseconds.
		Weight::from_parts(29_726_000, 0)
			.saturating_add(Weight::from_parts(0, 69033))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: BridgeDarwiniaMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeDarwiniaMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgePolkadotGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgePolkadotGrandpa ImportedHeaders (max_values: None, max_size: Some(65568), added: 68043, mode: MaxEncodedLen)
	/// Storage: BridgeDarwiniaMessages OutboundLanes (r:1 w:1)
	/// Proof: BridgeDarwiniaMessages OutboundLanes (max_values: None, max_size: Some(44), added: 2519, mode: MaxEncodedLen)
	/// Storage: DarwiniaFeeMarket Orders (r:1 w:0)
	/// Proof Skipped: DarwiniaFeeMarket Orders (max_values: None, max_size: None, mode: Measured)
	fn receive_delivery_proof_for_single_message() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `694`
		//  Estimated: `69033`
		// Minimum execution time: 22_913_000 picoseconds.
		Weight::from_parts(23_589_000, 0)
			.saturating_add(Weight::from_parts(0, 69033))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: BridgeDarwiniaMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeDarwiniaMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgePolkadotGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgePolkadotGrandpa ImportedHeaders (max_values: None, max_size: Some(65568), added: 68043, mode: MaxEncodedLen)
	/// Storage: BridgeDarwiniaMessages OutboundLanes (r:1 w:1)
	/// Proof: BridgeDarwiniaMessages OutboundLanes (max_values: None, max_size: Some(44), added: 2519, mode: MaxEncodedLen)
	/// Storage: DarwiniaFeeMarket Orders (r:2 w:0)
	/// Proof Skipped: DarwiniaFeeMarket Orders (max_values: None, max_size: None, mode: Measured)
	fn receive_delivery_proof_for_two_messages_by_single_relayer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `694`
		//  Estimated: `69033`
		// Minimum execution time: 24_934_000 picoseconds.
		Weight::from_parts(25_597_000, 0)
			.saturating_add(Weight::from_parts(0, 69033))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: BridgeDarwiniaMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeDarwiniaMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgePolkadotGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgePolkadotGrandpa ImportedHeaders (max_values: None, max_size: Some(65568), added: 68043, mode: MaxEncodedLen)
	/// Storage: BridgeDarwiniaMessages OutboundLanes (r:1 w:1)
	/// Proof: BridgeDarwiniaMessages OutboundLanes (max_values: None, max_size: Some(44), added: 2519, mode: MaxEncodedLen)
	/// Storage: DarwiniaFeeMarket Orders (r:2 w:0)
	/// Proof Skipped: DarwiniaFeeMarket Orders (max_values: None, max_size: None, mode: Measured)
	fn receive_delivery_proof_for_two_messages_by_two_relayers() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `694`
		//  Estimated: `69033`
		// Minimum execution time: 25_322_000 picoseconds.
		Weight::from_parts(26_072_000, 0)
			.saturating_add(Weight::from_parts(0, 69033))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
