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

//! Autogenerated weights for `pallet_collective`
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

/// Weight functions for `pallet_collective`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_collective::WeightInfo for WeightInfo<T> {
	// Storage: TechnicalCommittee Members (r:1 w:1)
	// Storage: TechnicalCommittee Proposals (r:1 w:0)
	// Storage: TechnicalCommittee Prime (r:0 w:1)
	// Storage: TechnicalCommittee Voting (r:100 w:100)
	/// The range of component `m` is `[0, 100]`.
	/// The range of component `n` is `[0, 100]`.
	/// The range of component `p` is `[0, 100]`.
	/// The range of component `m` is `[0, 100]`.
	/// The range of component `n` is `[0, 100]`.
	/// The range of component `p` is `[0, 100]`.
	fn set_members(m: u32, _n: u32, p: u32, ) -> Weight {
		// Minimum execution time: 13_463 nanoseconds.
		Weight::from_ref_time(13_463_000)
			// Standard Error: 895_060
			.saturating_add(Weight::from_ref_time(4_307_995).saturating_mul(m.into()))
			// Standard Error: 895_060
			.saturating_add(Weight::from_ref_time(4_273_975).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(m.into())))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(p.into())))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(m.into())))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(p.into())))
	}
	// Storage: TechnicalCommittee Members (r:1 w:0)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn execute(_b: u32, m: u32, ) -> Weight {
		// Minimum execution time: 18_470 nanoseconds.
		Weight::from_ref_time(19_245_456)
			// Standard Error: 9_898
			.saturating_add(Weight::from_ref_time(15_368).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	// Storage: TechnicalCommittee Members (r:1 w:0)
	// Storage: TechnicalCommittee ProposalOf (r:1 w:0)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn propose_execute(b: u32, _m: u32, ) -> Weight {
		// Minimum execution time: 21_278 nanoseconds.
		Weight::from_ref_time(22_250_858)
			// Standard Error: 2_161
			.saturating_add(Weight::from_ref_time(1_654).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(2))
	}
	// Storage: TechnicalCommittee Members (r:1 w:0)
	// Storage: TechnicalCommittee ProposalOf (r:1 w:1)
	// Storage: TechnicalCommittee Proposals (r:1 w:1)
	// Storage: TechnicalCommittee ProposalCount (r:1 w:1)
	// Storage: TechnicalCommittee Voting (r:0 w:1)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[2, 100]`.
	/// The range of component `p` is `[1, 100]`.
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[2, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn propose_proposed(_b: u32, m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 26_624 nanoseconds.
		Weight::from_ref_time(32_505_921)
			// Standard Error: 50_356
			.saturating_add(Weight::from_ref_time(46_724).saturating_mul(m.into()))
			// Standard Error: 49_848
			.saturating_add(Weight::from_ref_time(138_267).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: TechnicalCommittee Members (r:1 w:0)
	// Storage: TechnicalCommittee Voting (r:1 w:1)
	/// The range of component `m` is `[5, 100]`.
	/// The range of component `m` is `[5, 100]`.
	fn vote(_m: u32, ) -> Weight {
		// Minimum execution time: 32_143 nanoseconds.
		Weight::from_ref_time(38_586_052)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: TechnicalCommittee Voting (r:1 w:1)
	// Storage: TechnicalCommittee Members (r:1 w:0)
	// Storage: TechnicalCommittee Proposals (r:1 w:1)
	// Storage: TechnicalCommittee ProposalOf (r:0 w:1)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_disapproved(m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 27_661 nanoseconds.
		Weight::from_ref_time(26_414_753)
			// Standard Error: 6_439
			.saturating_add(Weight::from_ref_time(14_572).saturating_mul(m.into()))
			// Standard Error: 6_244
			.saturating_add(Weight::from_ref_time(203_454).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: TechnicalCommittee Voting (r:1 w:1)
	// Storage: TechnicalCommittee Members (r:1 w:0)
	// Storage: TechnicalCommittee ProposalOf (r:1 w:1)
	// Storage: TechnicalCommittee Proposals (r:1 w:1)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_approved(b: u32, _m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 41_390 nanoseconds.
		Weight::from_ref_time(40_470_154)
			// Standard Error: 4_809
			.saturating_add(Weight::from_ref_time(1_781).saturating_mul(b.into()))
			// Standard Error: 49_650
			.saturating_add(Weight::from_ref_time(225_335).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: TechnicalCommittee Voting (r:1 w:1)
	// Storage: TechnicalCommittee Members (r:1 w:0)
	// Storage: TechnicalCommittee Prime (r:1 w:0)
	// Storage: TechnicalCommittee Proposals (r:1 w:1)
	// Storage: TechnicalCommittee ProposalOf (r:0 w:1)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_disapproved(m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 31_571 nanoseconds.
		Weight::from_ref_time(31_010_232)
			// Standard Error: 4_721
			.saturating_add(Weight::from_ref_time(5_286).saturating_mul(m.into()))
			// Standard Error: 4_578
			.saturating_add(Weight::from_ref_time(178_121).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: TechnicalCommittee Voting (r:1 w:1)
	// Storage: TechnicalCommittee Members (r:1 w:0)
	// Storage: TechnicalCommittee Prime (r:1 w:0)
	// Storage: TechnicalCommittee ProposalOf (r:1 w:1)
	// Storage: TechnicalCommittee Proposals (r:1 w:1)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Minimum execution time: 40_552 nanoseconds.
		Weight::from_ref_time(33_111_820)
			// Standard Error: 2_568
			.saturating_add(Weight::from_ref_time(3_905).saturating_mul(b.into()))
			// Standard Error: 27_340
			.saturating_add(Weight::from_ref_time(34_500).saturating_mul(m.into()))
			// Standard Error: 26_512
			.saturating_add(Weight::from_ref_time(261_368).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: TechnicalCommittee Proposals (r:1 w:1)
	// Storage: TechnicalCommittee Voting (r:0 w:1)
	// Storage: TechnicalCommittee ProposalOf (r:0 w:1)
	/// The range of component `p` is `[1, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn disapprove_proposal(p: u32, ) -> Weight {
		// Minimum execution time: 18_178 nanoseconds.
		Weight::from_ref_time(18_225_823)
			// Standard Error: 5_019
			.saturating_add(Weight::from_ref_time(189_676).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(3))
	}
}