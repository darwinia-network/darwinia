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
//! DATE: 2023-05-25, STEPS: `2`, REPEAT: `1`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `cypress`, CPU: `AMD Ryzen 7 5700G with Radeon Graphics`
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
// pallet_bridge_messages

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_bridge_messages`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_bridge_messages::WeightInfo for WeightInfo<T> {
	/// Storage: BridgeCrabMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeCrabMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgeCrabMessages OutboundLanes (r:1 w:1)
	/// Proof: BridgeCrabMessages OutboundLanes (max_values: None, max_size: Some(44), added: 2519, mode: MaxEncodedLen)
	/// Storage: System Account (r:2 w:2)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	/// Storage: CrabFeeMarket AssignedRelayers (r:1 w:0)
	/// Proof Skipped: CrabFeeMarket AssignedRelayers (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: BridgeCrabMessages OutboundMessages (r:0 w:9)
	/// Proof: BridgeCrabMessages OutboundMessages (max_values: None, max_size: Some(2621484), added: 2623959, mode: MaxEncodedLen)
	fn send_minimal_message_worst_case() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `372`
		//  Estimated: `13025`
		// Minimum execution time: 108_256_000 picoseconds.
		Weight::from_parts(108_256_000, 0)
			.saturating_add(Weight::from_parts(0, 13025))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(12))
	}
	/// Storage: BridgeCrabMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeCrabMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgeCrabMessages OutboundLanes (r:1 w:1)
	/// Proof: BridgeCrabMessages OutboundLanes (max_values: None, max_size: Some(44), added: 2519, mode: MaxEncodedLen)
	/// Storage: System Account (r:2 w:2)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	/// Storage: CrabFeeMarket AssignedRelayers (r:1 w:0)
	/// Proof Skipped: CrabFeeMarket AssignedRelayers (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: BridgeCrabMessages OutboundMessages (r:0 w:9)
	/// Proof: BridgeCrabMessages OutboundMessages (max_values: None, max_size: Some(2621484), added: 2623959, mode: MaxEncodedLen)
	fn send_1_kb_message_worst_case() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `372`
		//  Estimated: `13025`
		// Minimum execution time: 106_229_000 picoseconds.
		Weight::from_parts(106_229_000, 0)
			.saturating_add(Weight::from_parts(0, 13025))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(12))
	}
	/// Storage: BridgeCrabMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeCrabMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgeCrabMessages OutboundLanes (r:1 w:1)
	/// Proof: BridgeCrabMessages OutboundLanes (max_values: None, max_size: Some(44), added: 2519, mode: MaxEncodedLen)
	/// Storage: System Account (r:2 w:2)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	/// Storage: CrabFeeMarket AssignedRelayers (r:1 w:0)
	/// Proof Skipped: CrabFeeMarket AssignedRelayers (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: BridgeCrabMessages OutboundMessages (r:0 w:9)
	/// Proof: BridgeCrabMessages OutboundMessages (max_values: None, max_size: Some(2621484), added: 2623959, mode: MaxEncodedLen)
	fn send_16_kb_message_worst_case() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `372`
		//  Estimated: `13025`
		// Minimum execution time: 130_814_000 picoseconds.
		Weight::from_parts(130_814_000, 0)
			.saturating_add(Weight::from_parts(0, 13025))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(12))
	}
	/// Storage: BridgeCrabMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeCrabMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgeKusamaGrandpa ImportedHeaders (max_values: None, max_size: Some(131104), added: 133579, mode: MaxEncodedLen)
	/// Storage: BridgeCrabMessages InboundLanes (r:1 w:1)
	/// Proof: BridgeCrabMessages InboundLanes (max_values: None, max_size: Some(5660), added: 8135, mode: MaxEncodedLen)
	fn receive_single_message_proof() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `542`
		//  Estimated: `145181`
		// Minimum execution time: 116_078_000 picoseconds.
		Weight::from_parts(116_078_000, 0)
			.saturating_add(Weight::from_parts(0, 145181))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: BridgeCrabMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeCrabMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgeKusamaGrandpa ImportedHeaders (max_values: None, max_size: Some(131104), added: 133579, mode: MaxEncodedLen)
	/// Storage: BridgeCrabMessages InboundLanes (r:1 w:1)
	/// Proof: BridgeCrabMessages InboundLanes (max_values: None, max_size: Some(5660), added: 8135, mode: MaxEncodedLen)
	fn receive_two_messages_proof() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `542`
		//  Estimated: `145181`
		// Minimum execution time: 144_574_000 picoseconds.
		Weight::from_parts(144_574_000, 0)
			.saturating_add(Weight::from_parts(0, 145181))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: BridgeCrabMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeCrabMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgeKusamaGrandpa ImportedHeaders (max_values: None, max_size: Some(131104), added: 133579, mode: MaxEncodedLen)
	/// Storage: BridgeCrabMessages InboundLanes (r:1 w:1)
	/// Proof: BridgeCrabMessages InboundLanes (max_values: None, max_size: Some(5660), added: 8135, mode: MaxEncodedLen)
	fn receive_single_message_proof_with_outbound_lane_state() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `542`
		//  Estimated: `145181`
		// Minimum execution time: 125_576_000 picoseconds.
		Weight::from_parts(125_576_000, 0)
			.saturating_add(Weight::from_parts(0, 145181))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: BridgeCrabMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeCrabMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgeKusamaGrandpa ImportedHeaders (max_values: None, max_size: Some(131104), added: 133579, mode: MaxEncodedLen)
	/// Storage: BridgeCrabMessages InboundLanes (r:1 w:1)
	/// Proof: BridgeCrabMessages InboundLanes (max_values: None, max_size: Some(5660), added: 8135, mode: MaxEncodedLen)
	fn receive_single_message_proof_1_kb() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `542`
		//  Estimated: `145181`
		// Minimum execution time: 122_084_000 picoseconds.
		Weight::from_parts(122_084_000, 0)
			.saturating_add(Weight::from_parts(0, 145181))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: BridgeCrabMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeCrabMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgeKusamaGrandpa ImportedHeaders (max_values: None, max_size: Some(131104), added: 133579, mode: MaxEncodedLen)
	/// Storage: BridgeCrabMessages InboundLanes (r:1 w:1)
	/// Proof: BridgeCrabMessages InboundLanes (max_values: None, max_size: Some(5660), added: 8135, mode: MaxEncodedLen)
	fn receive_single_message_proof_16_kb() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `542`
		//  Estimated: `145181`
		// Minimum execution time: 259_045_000 picoseconds.
		Weight::from_parts(259_045_000, 0)
			.saturating_add(Weight::from_parts(0, 145181))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: BridgeCrabMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeCrabMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgeKusamaGrandpa ImportedHeaders (max_values: None, max_size: Some(131104), added: 133579, mode: MaxEncodedLen)
	/// Storage: BridgeCrabMessages InboundLanes (r:1 w:1)
	/// Proof: BridgeCrabMessages InboundLanes (max_values: None, max_size: Some(5660), added: 8135, mode: MaxEncodedLen)
	fn receive_single_prepaid_message_proof() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `542`
		//  Estimated: `145181`
		// Minimum execution time: 72_496_000 picoseconds.
		Weight::from_parts(72_496_000, 0)
			.saturating_add(Weight::from_parts(0, 145181))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: BridgeCrabMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeCrabMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgeKusamaGrandpa ImportedHeaders (max_values: None, max_size: Some(131104), added: 133579, mode: MaxEncodedLen)
	/// Storage: BridgeCrabMessages OutboundLanes (r:1 w:1)
	/// Proof: BridgeCrabMessages OutboundLanes (max_values: None, max_size: Some(44), added: 2519, mode: MaxEncodedLen)
	/// Storage: CrabFeeMarket Orders (r:1 w:0)
	/// Proof Skipped: CrabFeeMarket Orders (max_values: None, max_size: None, mode: Measured)
	fn receive_delivery_proof_for_single_message() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `593`
		//  Estimated: `143623`
		// Minimum execution time: 57_759_000 picoseconds.
		Weight::from_parts(57_759_000, 0)
			.saturating_add(Weight::from_parts(0, 143623))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: BridgeCrabMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeCrabMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgeKusamaGrandpa ImportedHeaders (max_values: None, max_size: Some(131104), added: 133579, mode: MaxEncodedLen)
	/// Storage: BridgeCrabMessages OutboundLanes (r:1 w:1)
	/// Proof: BridgeCrabMessages OutboundLanes (max_values: None, max_size: Some(44), added: 2519, mode: MaxEncodedLen)
	/// Storage: CrabFeeMarket Orders (r:2 w:0)
	/// Proof Skipped: CrabFeeMarket Orders (max_values: None, max_size: None, mode: Measured)
	fn receive_delivery_proof_for_two_messages_by_single_relayer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `593`
		//  Estimated: `146098`
		// Minimum execution time: 64_884_000 picoseconds.
		Weight::from_parts(64_884_000, 0)
			.saturating_add(Weight::from_parts(0, 146098))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: BridgeCrabMessages PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeCrabMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2), added: 497, mode: MaxEncodedLen)
	/// Storage: BridgeKusamaGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgeKusamaGrandpa ImportedHeaders (max_values: None, max_size: Some(131104), added: 133579, mode: MaxEncodedLen)
	/// Storage: BridgeCrabMessages OutboundLanes (r:1 w:1)
	/// Proof: BridgeCrabMessages OutboundLanes (max_values: None, max_size: Some(44), added: 2519, mode: MaxEncodedLen)
	/// Storage: CrabFeeMarket Orders (r:2 w:0)
	/// Proof Skipped: CrabFeeMarket Orders (max_values: None, max_size: None, mode: Measured)
	fn receive_delivery_proof_for_two_messages_by_two_relayers() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `593`
		//  Estimated: `146098`
		// Minimum execution time: 65_791_000 picoseconds.
		Weight::from_parts(65_791_000, 0)
			.saturating_add(Weight::from_parts(0, 146098))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}