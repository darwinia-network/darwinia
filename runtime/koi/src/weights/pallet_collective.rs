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

//! Autogenerated weights for `pallet_collective`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 32.0.0
//! DATE: 2024-06-27, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("koi-dev")`, DB CACHE: 1024

// Executed Command:
// target/release/darwinia
// benchmark
// pallet
// --header
// .maintain/license-header
// --heap-pages
// 4096
// --chain
// koi-dev
// --output
// runtime/koi/src/weights
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

/// Weight functions for `pallet_collective`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_collective::WeightInfo for WeightInfo<T> {
	/// Storage: `TechnicalCommittee::Members` (r:1 w:1)
	/// Proof: `TechnicalCommittee::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::Proposals` (r:1 w:0)
	/// Proof: `TechnicalCommittee::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::Voting` (r:100 w:100)
	/// Proof: `TechnicalCommittee::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::Prime` (r:0 w:1)
	/// Proof: `TechnicalCommittee::Prime` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `m` is `[0, 100]`.
	/// The range of component `n` is `[0, 100]`.
	/// The range of component `p` is `[0, 100]`.
	fn set_members(m: u32, _n: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0 + m * (2021 ±0) + p * (2026 ±0)`
		//  Estimated: `12200 + m * (1231 ±15) + p * (3660 ±15)`
		// Minimum execution time: 11_000_000 picoseconds.
		Weight::from_parts(11_000_000, 0)
			.saturating_add(Weight::from_parts(0, 12200))
			// Standard Error: 70_896
			.saturating_add(Weight::from_parts(3_167_004, 0).saturating_mul(m.into()))
			// Standard Error: 70_896
			.saturating_add(Weight::from_parts(4_540_655, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(p.into())))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(p.into())))
			.saturating_add(Weight::from_parts(0, 1231).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 3660).saturating_mul(p.into()))
	}
	/// Storage: `TechnicalCommittee::Members` (r:1 w:0)
	/// Proof: `TechnicalCommittee::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TxPause::PausedCalls` (r:1 w:0)
	/// Proof: `TxPause::PausedCalls` (`max_values`: None, `max_size`: Some(532), added: 3007, mode: `MaxEncodedLen`)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn execute(b: u32, m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `73 + m * (20 ±0)`
		//  Estimated: `3997 + m * (20 ±0)`
		// Minimum execution time: 14_000_000 picoseconds.
		Weight::from_parts(14_244_966, 0)
			.saturating_add(Weight::from_parts(0, 3997))
			// Standard Error: 40
			.saturating_add(Weight::from_parts(728, 0).saturating_mul(b.into()))
			// Standard Error: 412
			.saturating_add(Weight::from_parts(2_042, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(Weight::from_parts(0, 20).saturating_mul(m.into()))
	}
	/// Storage: `TechnicalCommittee::Members` (r:1 w:0)
	/// Proof: `TechnicalCommittee::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::ProposalOf` (r:1 w:0)
	/// Proof: `TechnicalCommittee::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `TxPause::PausedCalls` (r:1 w:0)
	/// Proof: `TxPause::PausedCalls` (`max_values`: None, `max_size`: Some(532), added: 3007, mode: `MaxEncodedLen`)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn propose_execute(b: u32, m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `73 + m * (20 ±0)`
		//  Estimated: `3997 + m * (20 ±0)`
		// Minimum execution time: 15_000_000 picoseconds.
		Weight::from_parts(15_664_697, 0)
			.saturating_add(Weight::from_parts(0, 3997))
			// Standard Error: 43
			.saturating_add(Weight::from_parts(1_057, 0).saturating_mul(b.into()))
			// Standard Error: 445
			.saturating_add(Weight::from_parts(5_419, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(Weight::from_parts(0, 20).saturating_mul(m.into()))
	}
	/// Storage: `TechnicalCommittee::Members` (r:1 w:0)
	/// Proof: `TechnicalCommittee::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::ProposalOf` (r:1 w:1)
	/// Proof: `TechnicalCommittee::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::Proposals` (r:1 w:1)
	/// Proof: `TechnicalCommittee::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::ProposalCount` (r:1 w:1)
	/// Proof: `TechnicalCommittee::ProposalCount` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::Voting` (r:0 w:1)
	/// Proof: `TechnicalCommittee::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[2, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn propose_proposed(b: u32, m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `357 + m * (20 ±0) + p * (36 ±0)`
		//  Estimated: `3751 + m * (21 ±0) + p * (36 ±0)`
		// Minimum execution time: 16_000_000 picoseconds.
		Weight::from_parts(18_038_161, 0)
			.saturating_add(Weight::from_parts(0, 3751))
			// Standard Error: 77
			.saturating_add(Weight::from_parts(1_328, 0).saturating_mul(b.into()))
			// Standard Error: 806
			.saturating_add(Weight::from_parts(9_593, 0).saturating_mul(m.into()))
			// Standard Error: 796
			.saturating_add(Weight::from_parts(116_562, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
			.saturating_add(Weight::from_parts(0, 21).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 36).saturating_mul(p.into()))
	}
	/// Storage: `TechnicalCommittee::Members` (r:1 w:0)
	/// Proof: `TechnicalCommittee::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::Voting` (r:1 w:1)
	/// Proof: `TechnicalCommittee::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `m` is `[5, 100]`.
	fn vote(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `832 + m * (40 ±0)`
		//  Estimated: `4296 + m * (40 ±0)`
		// Minimum execution time: 14_000_000 picoseconds.
		Weight::from_parts(15_243_118, 0)
			.saturating_add(Weight::from_parts(0, 4296))
			// Standard Error: 729
			.saturating_add(Weight::from_parts(9_936, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(Weight::from_parts(0, 40).saturating_mul(m.into()))
	}
	/// Storage: `TechnicalCommittee::Voting` (r:1 w:1)
	/// Proof: `TechnicalCommittee::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::Members` (r:1 w:0)
	/// Proof: `TechnicalCommittee::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::Proposals` (r:1 w:1)
	/// Proof: `TechnicalCommittee::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::ProposalOf` (r:0 w:1)
	/// Proof: `TechnicalCommittee::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_disapproved(m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `409 + m * (40 ±0) + p * (36 ±0)`
		//  Estimated: `3854 + m * (41 ±0) + p * (36 ±0)`
		// Minimum execution time: 19_000_000 picoseconds.
		Weight::from_parts(19_230_367, 0)
			.saturating_add(Weight::from_parts(0, 3854))
			// Standard Error: 625
			.saturating_add(Weight::from_parts(3_816, 0).saturating_mul(m.into()))
			// Standard Error: 609
			.saturating_add(Weight::from_parts(119_137, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 41).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 36).saturating_mul(p.into()))
	}
	/// Storage: `TechnicalCommittee::Voting` (r:1 w:1)
	/// Proof: `TechnicalCommittee::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::Members` (r:1 w:0)
	/// Proof: `TechnicalCommittee::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::ProposalOf` (r:1 w:1)
	/// Proof: `TechnicalCommittee::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `TxPause::PausedCalls` (r:1 w:0)
	/// Proof: `TxPause::PausedCalls` (`max_values`: None, `max_size`: Some(532), added: 3007, mode: `MaxEncodedLen`)
	/// Storage: `TechnicalCommittee::Proposals` (r:1 w:1)
	/// Proof: `TechnicalCommittee::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `715 + b * (1 ±0) + m * (40 ±0) + p * (40 ±0)`
		//  Estimated: `4032 + b * (1 ±0) + m * (42 ±0) + p * (40 ±0)`
		// Minimum execution time: 30_000_000 picoseconds.
		Weight::from_parts(29_705_330, 0)
			.saturating_add(Weight::from_parts(0, 4032))
			// Standard Error: 123
			.saturating_add(Weight::from_parts(1_678, 0).saturating_mul(b.into()))
			// Standard Error: 1_301
			.saturating_add(Weight::from_parts(10_568, 0).saturating_mul(m.into()))
			// Standard Error: 1_268
			.saturating_add(Weight::from_parts(142_999, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 1).saturating_mul(b.into()))
			.saturating_add(Weight::from_parts(0, 42).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 40).saturating_mul(p.into()))
	}
	/// Storage: `TechnicalCommittee::Voting` (r:1 w:1)
	/// Proof: `TechnicalCommittee::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::Members` (r:1 w:0)
	/// Proof: `TechnicalCommittee::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::Prime` (r:1 w:0)
	/// Proof: `TechnicalCommittee::Prime` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::Proposals` (r:1 w:1)
	/// Proof: `TechnicalCommittee::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::ProposalOf` (r:0 w:1)
	/// Proof: `TechnicalCommittee::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_disapproved(m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `429 + m * (40 ±0) + p * (36 ±0)`
		//  Estimated: `3874 + m * (41 ±0) + p * (36 ±0)`
		// Minimum execution time: 20_000_000 picoseconds.
		Weight::from_parts(21_064_164, 0)
			.saturating_add(Weight::from_parts(0, 3874))
			// Standard Error: 853
			.saturating_add(Weight::from_parts(6_630, 0).saturating_mul(m.into()))
			// Standard Error: 832
			.saturating_add(Weight::from_parts(118_190, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 41).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 36).saturating_mul(p.into()))
	}
	/// Storage: `TechnicalCommittee::Voting` (r:1 w:1)
	/// Proof: `TechnicalCommittee::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::Members` (r:1 w:0)
	/// Proof: `TechnicalCommittee::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::Prime` (r:1 w:0)
	/// Proof: `TechnicalCommittee::Prime` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::ProposalOf` (r:1 w:1)
	/// Proof: `TechnicalCommittee::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `TxPause::PausedCalls` (r:1 w:0)
	/// Proof: `TxPause::PausedCalls` (`max_values`: None, `max_size`: Some(532), added: 3007, mode: `MaxEncodedLen`)
	/// Storage: `TechnicalCommittee::Proposals` (r:1 w:1)
	/// Proof: `TechnicalCommittee::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `735 + b * (1 ±0) + m * (40 ±0) + p * (40 ±0)`
		//  Estimated: `4052 + b * (1 ±0) + m * (42 ±0) + p * (40 ±0)`
		// Minimum execution time: 32_000_000 picoseconds.
		Weight::from_parts(34_768_602, 0)
			.saturating_add(Weight::from_parts(0, 4052))
			// Standard Error: 108
			.saturating_add(Weight::from_parts(362, 0).saturating_mul(b.into()))
			// Standard Error: 1_114
			.saturating_add(Weight::from_parts(134_128, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 1).saturating_mul(b.into()))
			.saturating_add(Weight::from_parts(0, 42).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 40).saturating_mul(p.into()))
	}
	/// Storage: `TechnicalCommittee::Proposals` (r:1 w:1)
	/// Proof: `TechnicalCommittee::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::Voting` (r:0 w:1)
	/// Proof: `TechnicalCommittee::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `TechnicalCommittee::ProposalOf` (r:0 w:1)
	/// Proof: `TechnicalCommittee::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `p` is `[1, 100]`.
	fn disapprove_proposal(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `226 + p * (32 ±0)`
		//  Estimated: `1711 + p * (32 ±0)`
		// Minimum execution time: 11_000_000 picoseconds.
		Weight::from_parts(12_143_115, 0)
			.saturating_add(Weight::from_parts(0, 1711))
			// Standard Error: 637
			.saturating_add(Weight::from_parts(103_056, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 32).saturating_mul(p.into()))
	}
}
