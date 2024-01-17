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

//! Autogenerated weights for `pallet_fee_market`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-01-17, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("pangolin-dev")`, DB CACHE: 1024

// Executed Command:
// target/release/darwinia
// benchmark
// pallet
// --header
// .maintain/license-header
// --heap-pages
// 4096
// --chain
// pangolin-dev
// --output
// runtime/pangolin/src/weights
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

/// Weight functions for `pallet_fee_market`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_fee_market::WeightInfo for WeightInfo<T> {
	/// Storage: `PangoroFeeMarket::Relayers` (r:1 w:1)
	/// Proof: `PangoroFeeMarket::Relayers` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(116), added: 2591, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Locks` (r:1 w:1)
	/// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1287), added: 3762, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Freezes` (r:1 w:0)
	/// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(37), added: 2512, mode: `MaxEncodedLen`)
	/// Storage: `PangoroFeeMarket::RelayersMap` (r:10 w:1)
	/// Proof: `PangoroFeeMarket::RelayersMap` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PangoroFeeMarket::Orders` (r:1 w:0)
	/// Proof: `PangoroFeeMarket::Orders` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PangoroFeeMarket::AssignedRelayersNumber` (r:1 w:0)
	/// Proof: `PangoroFeeMarket::AssignedRelayersNumber` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PangoroFeeMarket::AssignedRelayers` (r:0 w:1)
	/// Proof: `PangoroFeeMarket::AssignedRelayers` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn enroll_and_lock_collateral() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1683`
		//  Estimated: `27423`
		// Minimum execution time: 110_000_000 picoseconds.
		Weight::from_parts(111_000_000, 0)
			.saturating_add(Weight::from_parts(0, 27423))
			.saturating_add(T::DbWeight::get().reads(16))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// Storage: `PangoroFeeMarket::Relayers` (r:1 w:0)
	/// Proof: `PangoroFeeMarket::Relayers` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(116), added: 2591, mode: `MaxEncodedLen`)
	/// Storage: `PangoroFeeMarket::RelayersMap` (r:10 w:1)
	/// Proof: `PangoroFeeMarket::RelayersMap` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Balances::Locks` (r:1 w:1)
	/// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1287), added: 3762, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Freezes` (r:1 w:0)
	/// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(37), added: 2512, mode: `MaxEncodedLen`)
	/// Storage: `PangoroFeeMarket::Orders` (r:1 w:0)
	/// Proof: `PangoroFeeMarket::Orders` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PangoroFeeMarket::AssignedRelayersNumber` (r:1 w:0)
	/// Proof: `PangoroFeeMarket::AssignedRelayersNumber` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PangoroFeeMarket::AssignedRelayers` (r:0 w:1)
	/// Proof: `PangoroFeeMarket::AssignedRelayers` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn increase_locked_collateral() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1755`
		//  Estimated: `27495`
		// Minimum execution time: 103_000_000 picoseconds.
		Weight::from_parts(105_000_000, 0)
			.saturating_add(Weight::from_parts(0, 27495))
			.saturating_add(T::DbWeight::get().reads(16))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: `PangoroFeeMarket::Relayers` (r:1 w:0)
	/// Proof: `PangoroFeeMarket::Relayers` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(116), added: 2591, mode: `MaxEncodedLen`)
	/// Storage: `PangoroFeeMarket::RelayersMap` (r:10 w:1)
	/// Proof: `PangoroFeeMarket::RelayersMap` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PangoroFeeMarket::Orders` (r:1 w:0)
	/// Proof: `PangoroFeeMarket::Orders` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Balances::Locks` (r:1 w:1)
	/// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1287), added: 3762, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Freezes` (r:1 w:0)
	/// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(37), added: 2512, mode: `MaxEncodedLen`)
	/// Storage: `PangoroFeeMarket::AssignedRelayersNumber` (r:1 w:0)
	/// Proof: `PangoroFeeMarket::AssignedRelayersNumber` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PangoroFeeMarket::AssignedRelayers` (r:0 w:1)
	/// Proof: `PangoroFeeMarket::AssignedRelayers` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn decrease_locked_collateral() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1755`
		//  Estimated: `27495`
		// Minimum execution time: 126_000_000 picoseconds.
		Weight::from_parts(128_000_000, 0)
			.saturating_add(Weight::from_parts(0, 27495))
			.saturating_add(T::DbWeight::get().reads(16))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: `PangoroFeeMarket::Relayers` (r:1 w:0)
	/// Proof: `PangoroFeeMarket::Relayers` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PangoroFeeMarket::RelayersMap` (r:10 w:1)
	/// Proof: `PangoroFeeMarket::RelayersMap` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PangoroFeeMarket::Orders` (r:1 w:0)
	/// Proof: `PangoroFeeMarket::Orders` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PangoroFeeMarket::AssignedRelayersNumber` (r:1 w:0)
	/// Proof: `PangoroFeeMarket::AssignedRelayersNumber` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PangoroFeeMarket::AssignedRelayers` (r:0 w:1)
	/// Proof: `PangoroFeeMarket::AssignedRelayers` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn update_relay_fee() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1336`
		//  Estimated: `27076`
		// Minimum execution time: 82_000_000 picoseconds.
		Weight::from_parts(84_000_000, 0)
			.saturating_add(Weight::from_parts(0, 27076))
			.saturating_add(T::DbWeight::get().reads(13))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `PangoroFeeMarket::Relayers` (r:1 w:1)
	/// Proof: `PangoroFeeMarket::Relayers` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PangoroFeeMarket::Orders` (r:1 w:0)
	/// Proof: `PangoroFeeMarket::Orders` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Balances::Locks` (r:1 w:1)
	/// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1287), added: 3762, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Freezes` (r:1 w:0)
	/// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(37), added: 2512, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(116), added: 2591, mode: `MaxEncodedLen`)
	/// Storage: `PangoroFeeMarket::AssignedRelayers` (r:1 w:1)
	/// Proof: `PangoroFeeMarket::AssignedRelayers` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PangoroFeeMarket::RelayersMap` (r:9 w:1)
	/// Proof: `PangoroFeeMarket::RelayersMap` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PangoroFeeMarket::AssignedRelayersNumber` (r:1 w:0)
	/// Proof: `PangoroFeeMarket::AssignedRelayersNumber` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn cancel_enrollment() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1822`
		//  Estimated: `25087`
		// Minimum execution time: 104_000_000 picoseconds.
		Weight::from_parts(106_000_000, 0)
			.saturating_add(Weight::from_parts(0, 25087))
			.saturating_add(T::DbWeight::get().reads(16))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// Storage: `PangoroFeeMarket::CollateralSlashProtect` (r:0 w:1)
	/// Proof: `PangoroFeeMarket::CollateralSlashProtect` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn set_slash_protect() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 6_000_000 picoseconds.
		Weight::from_parts(6_000_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `PangoroFeeMarket::Relayers` (r:1 w:0)
	/// Proof: `PangoroFeeMarket::Relayers` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PangoroFeeMarket::RelayersMap` (r:10 w:0)
	/// Proof: `PangoroFeeMarket::RelayersMap` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PangoroFeeMarket::Orders` (r:1 w:0)
	/// Proof: `PangoroFeeMarket::Orders` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PangoroFeeMarket::AssignedRelayers` (r:0 w:1)
	/// Proof: `PangoroFeeMarket::AssignedRelayers` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PangoroFeeMarket::AssignedRelayersNumber` (r:0 w:1)
	/// Proof: `PangoroFeeMarket::AssignedRelayersNumber` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn set_assigned_relayers_number() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1336`
		//  Estimated: `27076`
		// Minimum execution time: 80_000_000 picoseconds.
		Weight::from_parts(82_000_000, 0)
			.saturating_add(Weight::from_parts(0, 27076))
			.saturating_add(T::DbWeight::get().reads(12))
			.saturating_add(T::DbWeight::get().writes(2))
	}
}
