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

//! Autogenerated weights for `pallet_elections_phragmen`
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

/// Weight functions for `pallet_elections_phragmen`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_elections_phragmen::WeightInfo for WeightInfo<T> {
	/// Storage: PhragmenElection Candidates (r:1 w:0)
	/// Proof Skipped: PhragmenElection Candidates (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PhragmenElection Members (r:1 w:0)
	/// Proof Skipped: PhragmenElection Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PhragmenElection RunnersUp (r:1 w:0)
	/// Proof Skipped: PhragmenElection RunnersUp (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PhragmenElection Voting (r:1 w:1)
	/// Proof Skipped: PhragmenElection Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: Balances Locks (r:1 w:1)
	/// Proof: Balances Locks (max_values: None, max_size: Some(1287), added: 3762, mode: MaxEncodedLen)
	/// The range of component `v` is `[1, 16]`.
	fn vote_equal(v: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `438 + v * (56 ±0)`
		//  Estimated: `14420 + v * (224 ±0)`
		// Minimum execution time: 19_396_000 picoseconds.
		Weight::from_parts(20_506_348, 0)
			.saturating_add(Weight::from_parts(0, 14420))
			// Standard Error: 6_780
			.saturating_add(Weight::from_parts(173_261, 0).saturating_mul(v.into()))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(Weight::from_parts(0, 224).saturating_mul(v.into()))
	}
	/// Storage: PhragmenElection Candidates (r:1 w:0)
	/// Proof Skipped: PhragmenElection Candidates (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PhragmenElection Members (r:1 w:0)
	/// Proof Skipped: PhragmenElection Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PhragmenElection RunnersUp (r:1 w:0)
	/// Proof Skipped: PhragmenElection RunnersUp (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PhragmenElection Voting (r:1 w:1)
	/// Proof Skipped: PhragmenElection Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: Balances Locks (r:1 w:1)
	/// Proof: Balances Locks (max_values: None, max_size: Some(1287), added: 3762, mode: MaxEncodedLen)
	/// The range of component `v` is `[2, 16]`.
	fn vote_more(v: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `418 + v * (56 ±0)`
		//  Estimated: `14344 + v * (224 ±0)`
		// Minimum execution time: 27_004_000 picoseconds.
		Weight::from_parts(28_198_270, 0)
			.saturating_add(Weight::from_parts(0, 14344))
			// Standard Error: 7_228
			.saturating_add(Weight::from_parts(166_278, 0).saturating_mul(v.into()))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(Weight::from_parts(0, 224).saturating_mul(v.into()))
	}
	/// Storage: PhragmenElection Candidates (r:1 w:0)
	/// Proof Skipped: PhragmenElection Candidates (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PhragmenElection Members (r:1 w:0)
	/// Proof Skipped: PhragmenElection Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PhragmenElection RunnersUp (r:1 w:0)
	/// Proof Skipped: PhragmenElection RunnersUp (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PhragmenElection Voting (r:1 w:1)
	/// Proof Skipped: PhragmenElection Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: Balances Locks (r:1 w:1)
	/// Proof: Balances Locks (max_values: None, max_size: Some(1287), added: 3762, mode: MaxEncodedLen)
	/// The range of component `v` is `[2, 16]`.
	fn vote_less(v: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `438 + v * (56 ±0)`
		//  Estimated: `14424 + v * (224 ±0)`
		// Minimum execution time: 27_143_000 picoseconds.
		Weight::from_parts(28_165_398, 0)
			.saturating_add(Weight::from_parts(0, 14424))
			// Standard Error: 13_203
			.saturating_add(Weight::from_parts(226_939, 0).saturating_mul(v.into()))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(Weight::from_parts(0, 224).saturating_mul(v.into()))
	}
	/// Storage: PhragmenElection Voting (r:1 w:1)
	/// Proof Skipped: PhragmenElection Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: Balances Locks (r:1 w:1)
	/// Proof: Balances Locks (max_values: None, max_size: Some(1287), added: 3762, mode: MaxEncodedLen)
	fn remove_voter() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `768`
		//  Estimated: `8985`
		// Minimum execution time: 25_650_000 picoseconds.
		Weight::from_parts(27_093_000, 0)
			.saturating_add(Weight::from_parts(0, 8985))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: PhragmenElection Candidates (r:1 w:1)
	/// Proof Skipped: PhragmenElection Candidates (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PhragmenElection Members (r:1 w:0)
	/// Proof Skipped: PhragmenElection Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PhragmenElection RunnersUp (r:1 w:0)
	/// Proof Skipped: PhragmenElection RunnersUp (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `c` is `[1, 30]`.
	fn submit_candidacy(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1149 + c * (36 ±0)`
		//  Estimated: `7875 + c * (111 ±0)`
		// Minimum execution time: 22_958_000 picoseconds.
		Weight::from_parts(25_157_675, 0)
			.saturating_add(Weight::from_parts(0, 7875))
			// Standard Error: 5_176
			.saturating_add(Weight::from_parts(56_100, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(Weight::from_parts(0, 111).saturating_mul(c.into()))
	}
	/// Storage: PhragmenElection Candidates (r:1 w:1)
	/// Proof Skipped: PhragmenElection Candidates (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `c` is `[1, 30]`.
	fn renounce_candidacy_candidate(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `408 + c * (38 ±0)`
		//  Estimated: `1890 + c * (39 ±0)`
		// Minimum execution time: 20_594_000 picoseconds.
		Weight::from_parts(22_446_039, 0)
			.saturating_add(Weight::from_parts(0, 1890))
			// Standard Error: 4_576
			.saturating_add(Weight::from_parts(64_683, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(Weight::from_parts(0, 39).saturating_mul(c.into()))
	}
	/// Storage: PhragmenElection Members (r:1 w:1)
	/// Proof Skipped: PhragmenElection Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PhragmenElection RunnersUp (r:1 w:1)
	/// Proof Skipped: PhragmenElection RunnersUp (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council Prime (r:1 w:1)
	/// Proof Skipped: Council Prime (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council Proposals (r:1 w:0)
	/// Proof Skipped: Council Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council Members (r:0 w:1)
	/// Proof Skipped: Council Members (max_values: Some(1), max_size: None, mode: Measured)
	fn renounce_candidacy_members() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1170`
		//  Estimated: `11790`
		// Minimum execution time: 30_345_000 picoseconds.
		Weight::from_parts(32_472_000, 0)
			.saturating_add(Weight::from_parts(0, 11790))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: PhragmenElection RunnersUp (r:1 w:1)
	/// Proof Skipped: PhragmenElection RunnersUp (max_values: Some(1), max_size: None, mode: Measured)
	fn renounce_candidacy_runners_up() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `733`
		//  Estimated: `2218`
		// Minimum execution time: 20_696_000 picoseconds.
		Weight::from_parts(21_870_000, 0)
			.saturating_add(Weight::from_parts(0, 2218))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Benchmark Override (r:0 w:0)
	/// Proof Skipped: Benchmark Override (max_values: None, max_size: None, mode: Measured)
	fn remove_member_without_replacement() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 500_000_000_000 picoseconds.
		Weight::from_parts(500_000_000_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// Storage: PhragmenElection Members (r:1 w:1)
	/// Proof Skipped: PhragmenElection Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: System Account (r:2 w:2)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	/// Storage: PhragmenElection RunnersUp (r:1 w:1)
	/// Proof Skipped: PhragmenElection RunnersUp (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council Prime (r:1 w:1)
	/// Proof Skipped: Council Prime (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council Proposals (r:1 w:0)
	/// Proof Skipped: Council Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council Members (r:0 w:1)
	/// Proof Skipped: Council Members (max_values: Some(1), max_size: None, mode: Measured)
	fn remove_member_with_replacement() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1365`
		//  Estimated: `18937`
		// Minimum execution time: 44_088_000 picoseconds.
		Weight::from_parts(47_529_000, 0)
			.saturating_add(Weight::from_parts(0, 18937))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	/// Storage: PhragmenElection Voting (r:301 w:300)
	/// Proof Skipped: PhragmenElection Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: PhragmenElection Members (r:1 w:0)
	/// Proof Skipped: PhragmenElection Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PhragmenElection RunnersUp (r:1 w:0)
	/// Proof Skipped: PhragmenElection RunnersUp (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PhragmenElection Candidates (r:1 w:0)
	/// Proof Skipped: PhragmenElection Candidates (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Balances Locks (r:300 w:300)
	/// Proof: Balances Locks (max_values: None, max_size: Some(1287), added: 3762, mode: MaxEncodedLen)
	/// Storage: System Account (r:300 w:300)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	/// The range of component `v` is `[150, 300]`.
	/// The range of component `d` is `[0, 150]`.
	fn clean_defunct_voters(v: u32, _d: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `973 + v * (582 ±0)`
		//  Estimated: `13872 + v * (11160 ±0)`
		// Minimum execution time: 6_491_318_000 picoseconds.
		Weight::from_parts(6_584_232_000, 0)
			.saturating_add(Weight::from_parts(0, 13872))
			// Standard Error: 180_782
			.saturating_add(Weight::from_parts(27_270_280, 0).saturating_mul(v.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().reads((3_u64).saturating_mul(v.into())))
			.saturating_add(T::DbWeight::get().writes((3_u64).saturating_mul(v.into())))
			.saturating_add(Weight::from_parts(0, 11160).saturating_mul(v.into()))
	}
	/// Storage: PhragmenElection Candidates (r:1 w:1)
	/// Proof Skipped: PhragmenElection Candidates (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PhragmenElection Members (r:1 w:1)
	/// Proof Skipped: PhragmenElection Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PhragmenElection RunnersUp (r:1 w:1)
	/// Proof Skipped: PhragmenElection RunnersUp (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: PhragmenElection Voting (r:301 w:0)
	/// Proof Skipped: PhragmenElection Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: Council Proposals (r:1 w:0)
	/// Proof Skipped: Council Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: System Account (r:17 w:17)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	/// Storage: PhragmenElection ElectionRounds (r:1 w:1)
	/// Proof Skipped: PhragmenElection ElectionRounds (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council Members (r:0 w:1)
	/// Proof Skipped: Council Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council Prime (r:0 w:1)
	/// Proof Skipped: Council Prime (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `c` is `[1, 30]`.
	/// The range of component `v` is `[1, 300]`.
	/// The range of component `e` is `[300, 4800]`.
	fn election_phragmen(c: u32, v: u32, e: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0 + c * (3085 ±0) + v * (400 ±0) + e * (18 ±0)`
		//  Estimated: `141720 + v * (3891 ±4) + e * (35 ±0) + c * (1738 ±10)`
		// Minimum execution time: 457_244_000 picoseconds.
		Weight::from_parts(465_630_000, 0)
			.saturating_add(Weight::from_parts(0, 141720))
			// Standard Error: 2_234_265
			.saturating_add(Weight::from_parts(13_230_443, 0).saturating_mul(c.into()))
			// Standard Error: 222_446
			.saturating_add(Weight::from_parts(6_815_234, 0).saturating_mul(v.into()))
			// Standard Error: 14_270
			.saturating_add(Weight::from_parts(171_505, 0).saturating_mul(e.into()))
			.saturating_add(T::DbWeight::get().reads(12))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(c.into())))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(v.into())))
			.saturating_add(T::DbWeight::get().writes(4))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(c.into())))
			.saturating_add(Weight::from_parts(0, 3891).saturating_mul(v.into()))
			.saturating_add(Weight::from_parts(0, 35).saturating_mul(e.into()))
			.saturating_add(Weight::from_parts(0, 1738).saturating_mul(c.into()))
	}
}
