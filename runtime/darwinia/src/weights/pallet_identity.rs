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
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("darwinia-dev"), DB CACHE: 1024

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
// darwinia-dev
// --output
// runtime/darwinia/src/weights
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
		// Minimum execution time: 7_842_000 picoseconds.
		Weight::from_parts(8_657_122, 0)
			.saturating_add(Weight::from_parts(0, 2386))
			// Standard Error: 2_905
			.saturating_add(Weight::from_parts(72_203, 0).saturating_mul(r.into()))
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
		// Minimum execution time: 22_838_000 picoseconds.
		Weight::from_parts(21_813_170, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 9_461
			.saturating_add(Weight::from_parts(202_158, 0).saturating_mul(r.into()))
			// Standard Error: 1_846
			.saturating_add(Weight::from_parts(385_111, 0).saturating_mul(x.into()))
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
		// Minimum execution time: 5_858_000 picoseconds.
		Weight::from_parts(16_930_663, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 4_903
			.saturating_add(Weight::from_parts(2_338_923, 0).saturating_mul(s.into()))
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
		// Minimum execution time: 5_844_000 picoseconds.
		Weight::from_parts(17_036_647, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 3_500
			.saturating_add(Weight::from_parts(939_863, 0).saturating_mul(p.into()))
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
		// Minimum execution time: 43_674_000 picoseconds.
		Weight::from_parts(23_675_746, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 13_124
			.saturating_add(Weight::from_parts(215_902, 0).saturating_mul(r.into()))
			// Standard Error: 2_562
			.saturating_add(Weight::from_parts(925_255, 0).saturating_mul(s.into()))
			// Standard Error: 2_562
			.saturating_add(Weight::from_parts(213_303, 0).saturating_mul(x.into()))
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
		// Minimum execution time: 23_277_000 picoseconds.
		Weight::from_parts(22_587_610, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 10_576
			.saturating_add(Weight::from_parts(117_101, 0).saturating_mul(r.into()))
			// Standard Error: 2_063
			.saturating_add(Weight::from_parts(395_625, 0).saturating_mul(x.into()))
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
		// Minimum execution time: 21_387_000 picoseconds.
		Weight::from_parts(20_046_699, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 7_089
			.saturating_add(Weight::from_parts(127_553, 0).saturating_mul(r.into()))
			// Standard Error: 1_383
			.saturating_add(Weight::from_parts(400_952, 0).saturating_mul(x.into()))
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
		// Minimum execution time: 4_757_000 picoseconds.
		Weight::from_parts(5_260_919, 0)
			.saturating_add(Weight::from_parts(0, 2386))
			// Standard Error: 1_385
			.saturating_add(Weight::from_parts(63_244, 0).saturating_mul(r.into()))
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
		// Minimum execution time: 4_463_000 picoseconds.
		Weight::from_parts(4_853_158, 0)
			.saturating_add(Weight::from_parts(0, 2386))
			// Standard Error: 1_456
			.saturating_add(Weight::from_parts(57_299, 0).saturating_mul(r.into()))
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
		// Minimum execution time: 4_175_000 picoseconds.
		Weight::from_parts(4_730_480, 0)
			.saturating_add(Weight::from_parts(0, 2386))
			// Standard Error: 1_155
			.saturating_add(Weight::from_parts(58_141, 0).saturating_mul(r.into()))
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
		// Minimum execution time: 14_354_000 picoseconds.
		Weight::from_parts(15_085_696, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 7_526
			.saturating_add(Weight::from_parts(77_444, 0).saturating_mul(r.into()))
			// Standard Error: 1_392
			.saturating_add(Weight::from_parts(617_748, 0).saturating_mul(x.into()))
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
		// Minimum execution time: 55_537_000 picoseconds.
		Weight::from_parts(38_109_879, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 15_110
			.saturating_add(Weight::from_parts(313_049, 0).saturating_mul(r.into()))
			// Standard Error: 2_950
			.saturating_add(Weight::from_parts(909_918, 0).saturating_mul(s.into()))
			// Standard Error: 2_950
			.saturating_add(Weight::from_parts(211_748, 0).saturating_mul(x.into()))
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
		// Minimum execution time: 19_235_000 picoseconds.
		Weight::from_parts(23_419_205, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 1_842
			.saturating_add(Weight::from_parts(51_166, 0).saturating_mul(s.into()))
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
		// Minimum execution time: 7_738_000 picoseconds.
		Weight::from_parts(9_492_097, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 1_144
			.saturating_add(Weight::from_parts(31_984, 0).saturating_mul(s.into()))
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
		// Minimum execution time: 22_609_000 picoseconds.
		Weight::from_parts(24_687_217, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 2_186
			.saturating_add(Weight::from_parts(58_758, 0).saturating_mul(s.into()))
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
		// Minimum execution time: 14_737_000 picoseconds.
		Weight::from_parts(16_956_281, 0)
			.saturating_add(Weight::from_parts(0, 5511))
			// Standard Error: 1_643
			.saturating_add(Weight::from_parts(49_574, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
}
