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
		//  Estimated: `13092`
		// Minimum execution time: 47_412_000 picoseconds.
		Weight::from_parts(48_953_000, 0)
			.saturating_add(Weight::from_parts(0, 13092))
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
		//  Estimated: `13092`
		// Minimum execution time: 47_978_000 picoseconds.
		Weight::from_parts(49_616_000, 0)
			.saturating_add(Weight::from_parts(0, 13092))
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
		//  Estimated: `13092`
		// Minimum execution time: 55_097_000 picoseconds.
		Weight::from_parts(58_194_000, 0)
			.saturating_add(Weight::from_parts(0, 13092))
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
		//  Estimated: `79645`
		// Minimum execution time: 29_399_000 picoseconds.
		Weight::from_parts(30_737_000, 0)
			.saturating_add(Weight::from_parts(0, 79645))
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
		//  Estimated: `79645`
		// Minimum execution time: 41_167_000 picoseconds.
		Weight::from_parts(42_489_000, 0)
			.saturating_add(Weight::from_parts(0, 79645))
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
		//  Estimated: `79645`
		// Minimum execution time: 33_541_000 picoseconds.
		Weight::from_parts(34_606_000, 0)
			.saturating_add(Weight::from_parts(0, 79645))
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
		//  Estimated: `79645`
		// Minimum execution time: 34_621_000 picoseconds.
		Weight::from_parts(36_057_000, 0)
			.saturating_add(Weight::from_parts(0, 79645))
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
		//  Estimated: `79645`
		// Minimum execution time: 72_042_000 picoseconds.
		Weight::from_parts(79_729_000, 0)
			.saturating_add(Weight::from_parts(0, 79645))
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
		//  Estimated: `79645`
		// Minimum execution time: 29_064_000 picoseconds.
		Weight::from_parts(30_609_000, 0)
			.saturating_add(Weight::from_parts(0, 79645))
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
		//  Estimated: `78188`
		// Minimum execution time: 23_785_000 picoseconds.
		Weight::from_parts(25_249_000, 0)
			.saturating_add(Weight::from_parts(0, 78188))
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
		//  Estimated: `80663`
		// Minimum execution time: 25_724_000 picoseconds.
		Weight::from_parts(26_348_000, 0)
			.saturating_add(Weight::from_parts(0, 80663))
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
		//  Estimated: `80663`
		// Minimum execution time: 25_977_000 picoseconds.
		Weight::from_parts(27_035_000, 0)
			.saturating_add(Weight::from_parts(0, 80663))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
