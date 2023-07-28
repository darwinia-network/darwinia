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
//! DATE: 2023-07-28, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `inv.cafe`, CPU: `13th Gen Intel(R) Core(TM) i9-13900K`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("pangoro-dev"), DB CACHE: 1024

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
// pangoro-dev
// --output
// runtime/pangoro/src/weights
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

/// Weight functions for `pallet_identity`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_identity::WeightInfo for WeightInfo<T> {
	/// Storage: Identity Registrars (r:1 w:1)
	/// Proof: Identity Registrars (max_values: Some(1), max_size: Some(901), added: 1396, mode: MaxEncodedLen)
	/// The range of component `r` is `[1, 19]`.
	fn add_registrar(r: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `32 + r * (45 ±0)`
		//  Estimated: `2386`
		// Minimum execution time: 7_781_000 picoseconds.
		Weight::from_parts(8_246_448, 0)
			.saturating_add(Weight::from_parts(0, 2386))
			// Standard Error: 2_386
			.saturating_add(Weight::from_parts(99_334, 0).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Identity IdentityOf (r:1 w:1)
	/// Proof: Identity IdentityOf (max_values: None, max_size: Some(7526), added: 10001, mode: MaxEncodedLen)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `x` is `[0, 100]`.
	fn set_identity(r: u32, x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `430 + r * (5 ±0)`
		//  Estimated: `10991`
		// Minimum execution time: 24_152_000 picoseconds.
		Weight::from_parts(21_871_050, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 9_244
			.saturating_add(Weight::from_parts(204_825, 0).saturating_mul(r.into()))
			// Standard Error: 1_803
			.saturating_add(Weight::from_parts(391_441, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Identity IdentityOf (r:1 w:0)
	/// Proof: Identity IdentityOf (max_values: None, max_size: Some(7526), added: 10001, mode: MaxEncodedLen)
	/// Storage: Identity SubsOf (r:1 w:1)
	/// Proof: Identity SubsOf (max_values: None, max_size: Some(2046), added: 4521, mode: MaxEncodedLen)
	/// Storage: Identity SuperOf (r:100 w:100)
	/// Proof: Identity SuperOf (max_values: None, max_size: Some(90), added: 2565, mode: MaxEncodedLen)
	/// The range of component `s` is `[0, 100]`.
	fn set_subs_new(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `89`
		//  Estimated: `10991 + s * (2565 ±0)`
		// Minimum execution time: 5_865_000 picoseconds.
		Weight::from_parts(18_769_987, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 26_789
			.saturating_add(Weight::from_parts(2_342_399, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(s.into())))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(s.into())))
			.saturating_add(Weight::from_parts(0, 2565).saturating_mul(s.into()))
	}
	/// Storage: Identity IdentityOf (r:1 w:0)
	/// Proof: Identity IdentityOf (max_values: None, max_size: Some(7526), added: 10001, mode: MaxEncodedLen)
	/// Storage: Identity SubsOf (r:1 w:1)
	/// Proof: Identity SubsOf (max_values: None, max_size: Some(2046), added: 4521, mode: MaxEncodedLen)
	/// Storage: Identity SuperOf (r:0 w:100)
	/// Proof: Identity SuperOf (max_values: None, max_size: Some(90), added: 2565, mode: MaxEncodedLen)
	/// The range of component `p` is `[0, 100]`.
	fn set_subs_old(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `169 + p * (20 ±0)`
		//  Estimated: `10991`
		// Minimum execution time: 5_742_000 picoseconds.
		Weight::from_parts(20_711_072, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 14_386
			.saturating_add(Weight::from_parts(901_991, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(p.into())))
	}
	/// Storage: Identity SubsOf (r:1 w:1)
	/// Proof: Identity SubsOf (max_values: None, max_size: Some(2046), added: 4521, mode: MaxEncodedLen)
	/// Storage: Identity IdentityOf (r:1 w:1)
	/// Proof: Identity IdentityOf (max_values: None, max_size: Some(7526), added: 10001, mode: MaxEncodedLen)
	/// Storage: Identity SuperOf (r:0 w:100)
	/// Proof: Identity SuperOf (max_values: None, max_size: Some(90), added: 2565, mode: MaxEncodedLen)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `s` is `[0, 100]`.
	/// The range of component `x` is `[0, 100]`.
	fn clear_identity(r: u32, s: u32, x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `445 + r * (5 ±0) + s * (20 ±0) + x * (66 ±0)`
		//  Estimated: `10991`
		// Minimum execution time: 43_941_000 picoseconds.
		Weight::from_parts(21_359_556, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 11_546
			.saturating_add(Weight::from_parts(318_979, 0).saturating_mul(r.into()))
			// Standard Error: 2_254
			.saturating_add(Weight::from_parts(939_441, 0).saturating_mul(s.into()))
			// Standard Error: 2_254
			.saturating_add(Weight::from_parts(213_708, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(s.into())))
	}
	/// Storage: Identity Registrars (r:1 w:0)
	/// Proof: Identity Registrars (max_values: Some(1), max_size: Some(901), added: 1396, mode: MaxEncodedLen)
	/// Storage: Identity IdentityOf (r:1 w:1)
	/// Proof: Identity IdentityOf (max_values: None, max_size: Some(7526), added: 10001, mode: MaxEncodedLen)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `x` is `[0, 100]`.
	fn request_judgement(r: u32, x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `355 + r * (45 ±0) + x * (66 ±0)`
		//  Estimated: `10991`
		// Minimum execution time: 23_566_000 picoseconds.
		Weight::from_parts(22_261_385, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 6_511
			.saturating_add(Weight::from_parts(160_287, 0).saturating_mul(r.into()))
			// Standard Error: 1_270
			.saturating_add(Weight::from_parts(392_954, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Identity IdentityOf (r:1 w:1)
	/// Proof: Identity IdentityOf (max_values: None, max_size: Some(7526), added: 10001, mode: MaxEncodedLen)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `x` is `[0, 100]`.
	fn cancel_request(r: u32, x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `386 + x * (66 ±0)`
		//  Estimated: `10991`
		// Minimum execution time: 21_689_000 picoseconds.
		Weight::from_parts(21_575_674, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 6_034
			.saturating_add(Weight::from_parts(72_626, 0).saturating_mul(r.into()))
			// Standard Error: 1_177
			.saturating_add(Weight::from_parts(391_389, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Identity Registrars (r:1 w:1)
	/// Proof: Identity Registrars (max_values: Some(1), max_size: Some(901), added: 1396, mode: MaxEncodedLen)
	/// The range of component `r` is `[1, 19]`.
	fn set_fee(r: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `77 + r * (45 ±0)`
		//  Estimated: `2386`
		// Minimum execution time: 4_833_000 picoseconds.
		Weight::from_parts(5_219_318, 0)
			.saturating_add(Weight::from_parts(0, 2386))
			// Standard Error: 1_271
			.saturating_add(Weight::from_parts(66_341, 0).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Identity Registrars (r:1 w:1)
	/// Proof: Identity Registrars (max_values: Some(1), max_size: Some(901), added: 1396, mode: MaxEncodedLen)
	/// The range of component `r` is `[1, 19]`.
	fn set_account_id(r: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `77 + r * (45 ±0)`
		//  Estimated: `2386`
		// Minimum execution time: 4_393_000 picoseconds.
		Weight::from_parts(4_852_549, 0)
			.saturating_add(Weight::from_parts(0, 2386))
			// Standard Error: 1_460
			.saturating_add(Weight::from_parts(54_802, 0).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Identity Registrars (r:1 w:1)
	/// Proof: Identity Registrars (max_values: Some(1), max_size: Some(901), added: 1396, mode: MaxEncodedLen)
	/// The range of component `r` is `[1, 19]`.
	fn set_fields(r: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `77 + r * (45 ±0)`
		//  Estimated: `2386`
		// Minimum execution time: 4_363_000 picoseconds.
		Weight::from_parts(4_828_626, 0)
			.saturating_add(Weight::from_parts(0, 2386))
			// Standard Error: 2_159
			.saturating_add(Weight::from_parts(54_287, 0).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Identity Registrars (r:1 w:0)
	/// Proof: Identity Registrars (max_values: Some(1), max_size: Some(901), added: 1396, mode: MaxEncodedLen)
	/// Storage: Identity IdentityOf (r:1 w:1)
	/// Proof: Identity IdentityOf (max_values: None, max_size: Some(7526), added: 10001, mode: MaxEncodedLen)
	/// The range of component `r` is `[1, 19]`.
	/// The range of component `x` is `[0, 100]`.
	fn provide_judgement(r: u32, x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `421 + r * (45 ±0) + x * (66 ±0)`
		//  Estimated: `10991`
		// Minimum execution time: 15_517_000 picoseconds.
		Weight::from_parts(14_489_953, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 5_377
			.saturating_add(Weight::from_parts(103_809, 0).saturating_mul(r.into()))
			// Standard Error: 994
			.saturating_add(Weight::from_parts(621_520, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Identity SubsOf (r:1 w:1)
	/// Proof: Identity SubsOf (max_values: None, max_size: Some(2046), added: 4521, mode: MaxEncodedLen)
	/// Storage: Identity IdentityOf (r:1 w:1)
	/// Proof: Identity IdentityOf (max_values: None, max_size: Some(7526), added: 10001, mode: MaxEncodedLen)
	/// Storage: System Account (r:2 w:2)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	/// Storage: Identity SuperOf (r:0 w:100)
	/// Proof: Identity SuperOf (max_values: None, max_size: Some(90), added: 2565, mode: MaxEncodedLen)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `s` is `[0, 100]`.
	/// The range of component `x` is `[0, 100]`.
	fn kill_identity(r: u32, s: u32, x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `675 + r * (11 ±0) + s * (20 ±0) + x * (66 ±0)`
		//  Estimated: `10991`
		// Minimum execution time: 59_112_000 picoseconds.
		Weight::from_parts(36_057_093, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 12_498
			.saturating_add(Weight::from_parts(354_110, 0).saturating_mul(r.into()))
			// Standard Error: 2_440
			.saturating_add(Weight::from_parts(939_053, 0).saturating_mul(s.into()))
			// Standard Error: 2_440
			.saturating_add(Weight::from_parts(220_332, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(s.into())))
	}
	/// Storage: Identity IdentityOf (r:1 w:0)
	/// Proof: Identity IdentityOf (max_values: None, max_size: Some(7526), added: 10001, mode: MaxEncodedLen)
	/// Storage: Identity SuperOf (r:1 w:1)
	/// Proof: Identity SuperOf (max_values: None, max_size: Some(90), added: 2565, mode: MaxEncodedLen)
	/// Storage: Identity SubsOf (r:1 w:1)
	/// Proof: Identity SubsOf (max_values: None, max_size: Some(2046), added: 4521, mode: MaxEncodedLen)
	/// The range of component `s` is `[0, 99]`.
	fn add_sub(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `536 + s * (23 ±0)`
		//  Estimated: `10991`
		// Minimum execution time: 20_482_000 picoseconds.
		Weight::from_parts(24_234_547, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 2_034
			.saturating_add(Weight::from_parts(44_475, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Identity IdentityOf (r:1 w:0)
	/// Proof: Identity IdentityOf (max_values: None, max_size: Some(7526), added: 10001, mode: MaxEncodedLen)
	/// Storage: Identity SuperOf (r:1 w:1)
	/// Proof: Identity SuperOf (max_values: None, max_size: Some(90), added: 2565, mode: MaxEncodedLen)
	/// The range of component `s` is `[1, 100]`.
	fn rename_sub(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `615 + s * (4 ±0)`
		//  Estimated: `10991`
		// Minimum execution time: 8_109_000 picoseconds.
		Weight::from_parts(9_611_643, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 765
			.saturating_add(Weight::from_parts(29_831, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Identity IdentityOf (r:1 w:0)
	/// Proof: Identity IdentityOf (max_values: None, max_size: Some(7526), added: 10001, mode: MaxEncodedLen)
	/// Storage: Identity SuperOf (r:1 w:1)
	/// Proof: Identity SuperOf (max_values: None, max_size: Some(90), added: 2565, mode: MaxEncodedLen)
	/// Storage: Identity SubsOf (r:1 w:1)
	/// Proof: Identity SubsOf (max_values: None, max_size: Some(2046), added: 4521, mode: MaxEncodedLen)
	/// The range of component `s` is `[1, 100]`.
	fn remove_sub(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `647 + s * (24 ±0)`
		//  Estimated: `10991`
		// Minimum execution time: 22_252_000 picoseconds.
		Weight::from_parts(24_820_757, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 2_038
			.saturating_add(Weight::from_parts(57_534, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Identity SuperOf (r:1 w:1)
	/// Proof: Identity SuperOf (max_values: None, max_size: Some(90), added: 2565, mode: MaxEncodedLen)
	/// Storage: Identity SubsOf (r:1 w:1)
	/// Proof: Identity SubsOf (max_values: None, max_size: Some(2046), added: 4521, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:0)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	/// The range of component `s` is `[0, 99]`.
	fn quit_sub(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `622 + s * (24 ±0)`
		//  Estimated: `5511`
		// Minimum execution time: 15_609_000 picoseconds.
		Weight::from_parts(17_474_012, 0)
			.saturating_add(Weight::from_parts(0, 5511))
			// Standard Error: 1_274
			.saturating_add(Weight::from_parts(43_068, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
}
