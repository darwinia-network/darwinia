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

//! Autogenerated weights for `pallet_conviction_voting`
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

/// Weight functions for `pallet_conviction_voting`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_conviction_voting::WeightInfo for WeightInfo<T> {
	/// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
	/// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(912), added: 3387, mode: `MaxEncodedLen`)
	/// Storage: `ConvictionVoting::VotingFor` (r:1 w:1)
	/// Proof: `ConvictionVoting::VotingFor` (`max_values`: None, `max_size`: Some(27229), added: 29704, mode: `MaxEncodedLen`)
	/// Storage: `ConvictionVoting::ClassLocksFor` (r:1 w:1)
	/// Proof: `ConvictionVoting::ClassLocksFor` (`max_values`: None, `max_size`: Some(119), added: 2594, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Locks` (r:1 w:1)
	/// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1287), added: 3762, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Freezes` (r:1 w:0)
	/// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(37), added: 2512, mode: `MaxEncodedLen`)
	/// Storage: `Scheduler::Agenda` (r:1 w:1)
	/// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
	fn vote_new() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `13189`
		//  Estimated: `42428`
		// Minimum execution time: 87_000_000 picoseconds.
		Weight::from_parts(95_000_000, 0)
			.saturating_add(Weight::from_parts(0, 42428))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
	/// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(912), added: 3387, mode: `MaxEncodedLen`)
	/// Storage: `ConvictionVoting::VotingFor` (r:1 w:1)
	/// Proof: `ConvictionVoting::VotingFor` (`max_values`: None, `max_size`: Some(27229), added: 29704, mode: `MaxEncodedLen`)
	/// Storage: `ConvictionVoting::ClassLocksFor` (r:1 w:1)
	/// Proof: `ConvictionVoting::ClassLocksFor` (`max_values`: None, `max_size`: Some(119), added: 2594, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Locks` (r:1 w:1)
	/// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1287), added: 3762, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Freezes` (r:1 w:0)
	/// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(37), added: 2512, mode: `MaxEncodedLen`)
	/// Storage: `Scheduler::Agenda` (r:2 w:2)
	/// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
	fn vote_existing() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `13910`
		//  Estimated: `83866`
		// Minimum execution time: 148_000_000 picoseconds.
		Weight::from_parts(158_000_000, 0)
			.saturating_add(Weight::from_parts(0, 83866))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	/// Storage: `ConvictionVoting::VotingFor` (r:1 w:1)
	/// Proof: `ConvictionVoting::VotingFor` (`max_values`: None, `max_size`: Some(27229), added: 29704, mode: `MaxEncodedLen`)
	/// Storage: `Referenda::ReferendumInfoFor` (r:1 w:1)
	/// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(912), added: 3387, mode: `MaxEncodedLen`)
	/// Storage: `Scheduler::Agenda` (r:2 w:2)
	/// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
	fn remove_vote() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `13659`
		//  Estimated: `83866`
		// Minimum execution time: 121_000_000 picoseconds.
		Weight::from_parts(126_000_000, 0)
			.saturating_add(Weight::from_parts(0, 83866))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: `ConvictionVoting::VotingFor` (r:1 w:1)
	/// Proof: `ConvictionVoting::VotingFor` (`max_values`: None, `max_size`: Some(27229), added: 29704, mode: `MaxEncodedLen`)
	/// Storage: `Referenda::ReferendumInfoFor` (r:1 w:0)
	/// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(912), added: 3387, mode: `MaxEncodedLen`)
	fn remove_other_vote() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `12783`
		//  Estimated: `30694`
		// Minimum execution time: 46_000_000 picoseconds.
		Weight::from_parts(51_000_000, 0)
			.saturating_add(Weight::from_parts(0, 30694))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `ConvictionVoting::VotingFor` (r:2 w:2)
	/// Proof: `ConvictionVoting::VotingFor` (`max_values`: None, `max_size`: Some(27229), added: 29704, mode: `MaxEncodedLen`)
	/// Storage: `Referenda::ReferendumInfoFor` (r:100 w:100)
	/// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(912), added: 3387, mode: `MaxEncodedLen`)
	/// Storage: `Scheduler::Agenda` (r:2 w:2)
	/// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
	/// Storage: `ConvictionVoting::ClassLocksFor` (r:1 w:1)
	/// Proof: `ConvictionVoting::ClassLocksFor` (`max_values`: None, `max_size`: Some(119), added: 2594, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Locks` (r:1 w:1)
	/// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1287), added: 3762, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Freezes` (r:1 w:0)
	/// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(37), added: 2512, mode: `MaxEncodedLen`)
	/// The range of component `r` is `[0, 100]`.
	fn delegate(r: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `7303 + r * (386 ±0)`
		//  Estimated: `83866 + r * (3387 ±0)`
		// Minimum execution time: 48_000_000 picoseconds.
		Weight::from_parts(29_493_210, 0)
			.saturating_add(Weight::from_parts(0, 83866))
			// Standard Error: 470_381
			.saturating_add(Weight::from_parts(55_692_665, 0).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(r.into())))
			.saturating_add(T::DbWeight::get().writes(6))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(r.into())))
			.saturating_add(Weight::from_parts(0, 3387).saturating_mul(r.into()))
	}
	/// Storage: `ConvictionVoting::VotingFor` (r:2 w:2)
	/// Proof: `ConvictionVoting::VotingFor` (`max_values`: None, `max_size`: Some(27229), added: 29704, mode: `MaxEncodedLen`)
	/// Storage: `Referenda::ReferendumInfoFor` (r:100 w:100)
	/// Proof: `Referenda::ReferendumInfoFor` (`max_values`: None, `max_size`: Some(912), added: 3387, mode: `MaxEncodedLen`)
	/// Storage: `Scheduler::Agenda` (r:2 w:2)
	/// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(38963), added: 41438, mode: `MaxEncodedLen`)
	/// The range of component `r` is `[0, 100]`.
	fn undelegate(r: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `7159 + r * (386 ±0)`
		//  Estimated: `83866 + r * (3387 ±0)`
		// Minimum execution time: 21_000_000 picoseconds.
		Weight::from_parts(1_474_467, 0)
			.saturating_add(Weight::from_parts(0, 83866))
			// Standard Error: 470_270
			.saturating_add(Weight::from_parts(55_664_610, 0).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(r.into())))
			.saturating_add(T::DbWeight::get().writes(4))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(r.into())))
			.saturating_add(Weight::from_parts(0, 3387).saturating_mul(r.into()))
	}
	/// Storage: `ConvictionVoting::VotingFor` (r:1 w:1)
	/// Proof: `ConvictionVoting::VotingFor` (`max_values`: None, `max_size`: Some(27229), added: 29704, mode: `MaxEncodedLen`)
	/// Storage: `ConvictionVoting::ClassLocksFor` (r:1 w:1)
	/// Proof: `ConvictionVoting::ClassLocksFor` (`max_values`: None, `max_size`: Some(119), added: 2594, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Locks` (r:1 w:1)
	/// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1287), added: 3762, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Freezes` (r:1 w:0)
	/// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(37), added: 2512, mode: `MaxEncodedLen`)
	fn unlock() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `11969`
		//  Estimated: `30694`
		// Minimum execution time: 73_000_000 picoseconds.
		Weight::from_parts(79_000_000, 0)
			.saturating_add(Weight::from_parts(0, 30694))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
}
