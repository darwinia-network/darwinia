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

//! Autogenerated weights for `pallet_identity`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-02-21, STEPS: `2`, REPEAT: 1, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `Debian`, CPU: `12th Gen Intel(R) Core(TM) i9-12900K`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("pangolin-local"), DB CACHE: 1024

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
// pangolin-local
// --output
// runtime/pangolin/src/weights/
// --extrinsic
// *
// --pallet
// *

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_identity`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_identity::WeightInfo for WeightInfo<T> {
	// Storage: Identity Registrars (r:1 w:1)
	/// The range of component `r` is `[1, 19]`.
	fn add_registrar(_r: u32, ) -> Weight {
		// Minimum execution time: 17_244 nanoseconds.
		Weight::from_parts(21_349_000, 0)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Identity IdentityOf (r:1 w:1)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `x` is `[0, 100]`.
	fn set_identity(_r: u32, x: u32, ) -> Weight {
		// Minimum execution time: 37_214 nanoseconds.
		Weight::from_parts(44_716_631, 0)
			// Standard Error: 32_727
			.saturating_add(Weight::from_parts(364_345, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Identity IdentityOf (r:1 w:0)
	// Storage: Identity SubsOf (r:1 w:1)
	// Storage: Identity SuperOf (r:100 w:100)
	/// The range of component `s` is `[0, 100]`.
	fn set_subs_new(_s: u32, ) -> Weight {
		// Minimum execution time: 15_193 nanoseconds.
		Weight::from_parts(269_849_000, 0)
			.saturating_add(T::DbWeight::get().reads(102))
			.saturating_add(T::DbWeight::get().writes(101))
	}
	// Storage: Identity IdentityOf (r:1 w:0)
	// Storage: Identity SubsOf (r:1 w:1)
	// Storage: Identity SuperOf (r:0 w:100)
	/// The range of component `p` is `[0, 100]`.
	fn set_subs_old(_p: u32, ) -> Weight {
		// Minimum execution time: 15_333 nanoseconds.
		Weight::from_parts(128_932_000, 0)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(101))
	}
	// Storage: Identity SubsOf (r:1 w:1)
	// Storage: Identity IdentityOf (r:1 w:1)
	// Storage: Identity SuperOf (r:0 w:100)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `s` is `[0, 100]`.
	/// The range of component `x` is `[0, 100]`.
	fn clear_identity(r: u32, s: u32, x: u32, ) -> Weight {
		// Minimum execution time: 53_139 nanoseconds.
		Weight::from_parts(34_638_684, 0)
			// Standard Error: 1_191_269
			.saturating_add(Weight::from_parts(105_649, 0).saturating_mul(r.into()))
			// Standard Error: 226_341
			.saturating_add(Weight::from_parts(1_081_613, 0).saturating_mul(s.into()))
			// Standard Error: 226_341
			.saturating_add(Weight::from_parts(163_873, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(s.into())))
	}
	// Storage: Identity Registrars (r:1 w:0)
	// Storage: Identity IdentityOf (r:1 w:1)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `x` is `[0, 100]`.
	fn request_judgement(r: u32, x: u32, ) -> Weight {
		// Minimum execution time: 36_771 nanoseconds.
		Weight::from_parts(34_392_578, 0)
			// Standard Error: 40_065
			.saturating_add(Weight::from_parts(118_921, 0).saturating_mul(r.into()))
			// Standard Error: 7_612
			.saturating_add(Weight::from_parts(364_635, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Identity IdentityOf (r:1 w:1)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `x` is `[0, 100]`.
	fn cancel_request(r: u32, x: u32, ) -> Weight {
		// Minimum execution time: 36_991 nanoseconds.
		Weight::from_parts(32_892_052, 0)
			// Standard Error: 536_206
			.saturating_add(Weight::from_parts(204_947, 0).saturating_mul(r.into()))
			// Standard Error: 101_879
			.saturating_add(Weight::from_parts(367_020, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Identity Registrars (r:1 w:1)
	/// The range of component `r` is `[1, 19]`.
	fn set_fee(_r: u32, ) -> Weight {
		// Minimum execution time: 10_489 nanoseconds.
		Weight::from_parts(12_110_000, 0)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Identity Registrars (r:1 w:1)
	/// The range of component `r` is `[1, 19]`.
	fn set_account_id(_r: u32, ) -> Weight {
		// Minimum execution time: 10_109 nanoseconds.
		Weight::from_parts(12_655_000, 0)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Identity Registrars (r:1 w:1)
	/// The range of component `r` is `[1, 19]`.
	fn set_fields(_r: u32, ) -> Weight {
		// Minimum execution time: 9_879 nanoseconds.
		Weight::from_parts(12_937_000, 0)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Identity Registrars (r:1 w:0)
	// Storage: Identity IdentityOf (r:1 w:1)
	/// The range of component `r` is `[1, 19]`.
	/// The range of component `x` is `[0, 100]`.
	fn provide_judgement(r: u32, x: u32, ) -> Weight {
		// Minimum execution time: 31_307 nanoseconds.
		Weight::from_parts(28_926_722, 0)
			// Standard Error: 637_875
			.saturating_add(Weight::from_parts(125_277, 0).saturating_mul(r.into()))
			// Standard Error: 114_817
			.saturating_add(Weight::from_parts(590_910, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Identity SubsOf (r:1 w:1)
	// Storage: Identity IdentityOf (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: Identity SuperOf (r:0 w:100)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `s` is `[0, 100]`.
	/// The range of component `x` is `[0, 100]`.
	fn kill_identity(r: u32, s: u32, x: u32, ) -> Weight {
		// Minimum execution time: 66_461 nanoseconds.
		Weight::from_parts(37_626_947, 0)
			// Standard Error: 303_300
			.saturating_add(Weight::from_parts(66_385, 0).saturating_mul(r.into()))
			// Standard Error: 57_627
			.saturating_add(Weight::from_parts(1_027_823, 0).saturating_mul(s.into()))
			// Standard Error: 57_627
			.saturating_add(Weight::from_parts(275_063, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(s.into())))
	}
	// Storage: Identity IdentityOf (r:1 w:0)
	// Storage: Identity SuperOf (r:1 w:1)
	// Storage: Identity SubsOf (r:1 w:1)
	/// The range of component `s` is `[0, 99]`.
	fn add_sub(_s: u32, ) -> Weight {
		// Minimum execution time: 40_637 nanoseconds.
		Weight::from_parts(41_225_000, 0)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Identity IdentityOf (r:1 w:0)
	// Storage: Identity SuperOf (r:1 w:1)
	/// The range of component `s` is `[1, 100]`.
	fn rename_sub(_s: u32, ) -> Weight {
		// Minimum execution time: 15_861 nanoseconds.
		Weight::from_parts(22_252_000, 0)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Identity IdentityOf (r:1 w:0)
	// Storage: Identity SuperOf (r:1 w:1)
	// Storage: Identity SubsOf (r:1 w:1)
	/// The range of component `s` is `[1, 100]`.
	fn remove_sub(_s: u32, ) -> Weight {
		// Minimum execution time: 41_624 nanoseconds.
		Weight::from_parts(46_038_000, 0)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Identity SuperOf (r:1 w:1)
	// Storage: Identity SubsOf (r:1 w:1)
	/// The range of component `s` is `[0, 99]`.
	fn quit_sub(_s: u32, ) -> Weight {
		// Minimum execution time: 25_832 nanoseconds.
		Weight::from_parts(34_866_000, 0)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
}
