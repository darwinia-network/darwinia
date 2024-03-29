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

//! Autogenerated weights for `pallet_identity`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-03-06, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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

/// Weight functions for `pallet_identity`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_identity::WeightInfo for WeightInfo<T> {
	/// Storage: `Identity::Registrars` (r:1 w:1)
	/// Proof: `Identity::Registrars` (`max_values`: Some(1), `max_size`: Some(901), added: 1396, mode: `MaxEncodedLen`)
	/// The range of component `r` is `[1, 19]`.
	fn add_registrar(r: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `31 + r * (45 ±0)`
		//  Estimated: `2386`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(9_176_591, 0)
			.saturating_add(Weight::from_parts(0, 2386))
			// Standard Error: 2_914
			.saturating_add(Weight::from_parts(61_492, 0).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Identity::IdentityOf` (r:1 w:1)
	/// Proof: `Identity::IdentityOf` (`max_values`: None, `max_size`: Some(7526), added: 10001, mode: `MaxEncodedLen`)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `x` is `[0, 100]`.
	fn set_identity(r: u32, x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `429 + r * (5 ±0)`
		//  Estimated: `10991`
		// Minimum execution time: 25_000_000 picoseconds.
		Weight::from_parts(24_650_388, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 3_218
			.saturating_add(Weight::from_parts(40_341, 0).saturating_mul(r.into()))
			// Standard Error: 627
			.saturating_add(Weight::from_parts(500_763, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Identity::IdentityOf` (r:1 w:0)
	/// Proof: `Identity::IdentityOf` (`max_values`: None, `max_size`: Some(7526), added: 10001, mode: `MaxEncodedLen`)
	/// Storage: `Identity::SubsOf` (r:1 w:1)
	/// Proof: `Identity::SubsOf` (`max_values`: None, `max_size`: Some(2046), added: 4521, mode: `MaxEncodedLen`)
	/// Storage: `Identity::SuperOf` (r:100 w:100)
	/// Proof: `Identity::SuperOf` (`max_values`: None, `max_size`: Some(90), added: 2565, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[0, 100]`.
	fn set_subs_new(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `88`
		//  Estimated: `10991 + s * (2565 ±0)`
		// Minimum execution time: 6_000_000 picoseconds.
		Weight::from_parts(17_207_871, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 2_644
			.saturating_add(Weight::from_parts(2_859_776, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(s.into())))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(s.into())))
			.saturating_add(Weight::from_parts(0, 2565).saturating_mul(s.into()))
	}
	/// Storage: `Identity::IdentityOf` (r:1 w:0)
	/// Proof: `Identity::IdentityOf` (`max_values`: None, `max_size`: Some(7526), added: 10001, mode: `MaxEncodedLen`)
	/// Storage: `Identity::SubsOf` (r:1 w:1)
	/// Proof: `Identity::SubsOf` (`max_values`: None, `max_size`: Some(2046), added: 4521, mode: `MaxEncodedLen`)
	/// Storage: `Identity::SuperOf` (r:0 w:100)
	/// Proof: `Identity::SuperOf` (`max_values`: None, `max_size`: Some(90), added: 2565, mode: `MaxEncodedLen`)
	/// The range of component `p` is `[0, 100]`.
	fn set_subs_old(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `168 + p * (20 ±0)`
		//  Estimated: `10991`
		// Minimum execution time: 6_000_000 picoseconds.
		Weight::from_parts(17_862_239, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 2_725
			.saturating_add(Weight::from_parts(1_163_444, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(p.into())))
	}
	/// Storage: `Identity::SubsOf` (r:1 w:1)
	/// Proof: `Identity::SubsOf` (`max_values`: None, `max_size`: Some(2046), added: 4521, mode: `MaxEncodedLen`)
	/// Storage: `Identity::IdentityOf` (r:1 w:1)
	/// Proof: `Identity::IdentityOf` (`max_values`: None, `max_size`: Some(7526), added: 10001, mode: `MaxEncodedLen`)
	/// Storage: `Identity::SuperOf` (r:0 w:100)
	/// Proof: `Identity::SuperOf` (`max_values`: None, `max_size`: Some(90), added: 2565, mode: `MaxEncodedLen`)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `s` is `[0, 100]`.
	/// The range of component `x` is `[0, 100]`.
	fn clear_identity(r: u32, s: u32, x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `444 + r * (5 ±0) + s * (20 ±0) + x * (66 ±0)`
		//  Estimated: `10991`
		// Minimum execution time: 50_000_000 picoseconds.
		Weight::from_parts(25_137_747, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 6_631
			.saturating_add(Weight::from_parts(38_735, 0).saturating_mul(r.into()))
			// Standard Error: 1_295
			.saturating_add(Weight::from_parts(1_147_063, 0).saturating_mul(s.into()))
			// Standard Error: 1_295
			.saturating_add(Weight::from_parts(258_600, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(s.into())))
	}
	/// Storage: `Identity::Registrars` (r:1 w:0)
	/// Proof: `Identity::Registrars` (`max_values`: Some(1), `max_size`: Some(901), added: 1396, mode: `MaxEncodedLen`)
	/// Storage: `Identity::IdentityOf` (r:1 w:1)
	/// Proof: `Identity::IdentityOf` (`max_values`: None, `max_size`: Some(7526), added: 10001, mode: `MaxEncodedLen`)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `x` is `[0, 100]`.
	fn request_judgement(r: u32, x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `354 + r * (45 ±0) + x * (66 ±0)`
		//  Estimated: `10991`
		// Minimum execution time: 25_000_000 picoseconds.
		Weight::from_parts(24_515_488, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 4_680
			.saturating_add(Weight::from_parts(51_675, 0).saturating_mul(r.into()))
			// Standard Error: 913
			.saturating_add(Weight::from_parts(510_043, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Identity::IdentityOf` (r:1 w:1)
	/// Proof: `Identity::IdentityOf` (`max_values`: None, `max_size`: Some(7526), added: 10001, mode: `MaxEncodedLen`)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `x` is `[0, 100]`.
	fn cancel_request(r: u32, x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `385 + x * (66 ±0)`
		//  Estimated: `10991`
		// Minimum execution time: 23_000_000 picoseconds.
		Weight::from_parts(22_170_119, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 4_466
			.saturating_add(Weight::from_parts(31_723, 0).saturating_mul(r.into()))
			// Standard Error: 871
			.saturating_add(Weight::from_parts(512_542, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Identity::Registrars` (r:1 w:1)
	/// Proof: `Identity::Registrars` (`max_values`: Some(1), `max_size`: Some(901), added: 1396, mode: `MaxEncodedLen`)
	/// The range of component `r` is `[1, 19]`.
	fn set_fee(r: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `76 + r * (45 ±0)`
		//  Estimated: `2386`
		// Minimum execution time: 5_000_000 picoseconds.
		Weight::from_parts(5_864_632, 0)
			.saturating_add(Weight::from_parts(0, 2386))
			// Standard Error: 3_416
			.saturating_add(Weight::from_parts(55_815, 0).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Identity::Registrars` (r:1 w:1)
	/// Proof: `Identity::Registrars` (`max_values`: Some(1), `max_size`: Some(901), added: 1396, mode: `MaxEncodedLen`)
	/// The range of component `r` is `[1, 19]`.
	fn set_account_id(r: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `76 + r * (45 ±0)`
		//  Estimated: `2386`
		// Minimum execution time: 5_000_000 picoseconds.
		Weight::from_parts(5_156_373, 0)
			.saturating_add(Weight::from_parts(0, 2386))
			// Standard Error: 2_792
			.saturating_add(Weight::from_parts(58_364, 0).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Identity::Registrars` (r:1 w:1)
	/// Proof: `Identity::Registrars` (`max_values`: Some(1), `max_size`: Some(901), added: 1396, mode: `MaxEncodedLen`)
	/// The range of component `r` is `[1, 19]`.
	fn set_fields(r: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `76 + r * (45 ±0)`
		//  Estimated: `2386`
		// Minimum execution time: 5_000_000 picoseconds.
		Weight::from_parts(5_187_523, 0)
			.saturating_add(Weight::from_parts(0, 2386))
			// Standard Error: 2_524
			.saturating_add(Weight::from_parts(59_083, 0).saturating_mul(r.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Identity::Registrars` (r:1 w:0)
	/// Proof: `Identity::Registrars` (`max_values`: Some(1), `max_size`: Some(901), added: 1396, mode: `MaxEncodedLen`)
	/// Storage: `Identity::IdentityOf` (r:1 w:1)
	/// Proof: `Identity::IdentityOf` (`max_values`: None, `max_size`: Some(7526), added: 10001, mode: `MaxEncodedLen`)
	/// The range of component `r` is `[1, 19]`.
	/// The range of component `x` is `[0, 100]`.
	fn provide_judgement(r: u32, x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `420 + r * (45 ±0) + x * (66 ±0)`
		//  Estimated: `10991`
		// Minimum execution time: 16_000_000 picoseconds.
		Weight::from_parts(15_416_905, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 4_310
			.saturating_add(Weight::from_parts(44_222, 0).saturating_mul(r.into()))
			// Standard Error: 797
			.saturating_add(Weight::from_parts(806_266, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Identity::SubsOf` (r:1 w:1)
	/// Proof: `Identity::SubsOf` (`max_values`: None, `max_size`: Some(2046), added: 4521, mode: `MaxEncodedLen`)
	/// Storage: `Identity::IdentityOf` (r:1 w:1)
	/// Proof: `Identity::IdentityOf` (`max_values`: None, `max_size`: Some(7526), added: 10001, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(116), added: 2591, mode: `MaxEncodedLen`)
	/// Storage: `Identity::SuperOf` (r:0 w:100)
	/// Proof: `Identity::SuperOf` (`max_values`: None, `max_size`: Some(90), added: 2565, mode: `MaxEncodedLen`)
	/// The range of component `r` is `[1, 20]`.
	/// The range of component `s` is `[0, 100]`.
	/// The range of component `x` is `[0, 100]`.
	fn kill_identity(r: u32, s: u32, x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `742 + r * (11 ±0) + s * (20 ±0) + x * (66 ±0)`
		//  Estimated: `10991`
		// Minimum execution time: 66_000_000 picoseconds.
		Weight::from_parts(40_972_968, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 8_013
			.saturating_add(Weight::from_parts(37_977, 0).saturating_mul(r.into()))
			// Standard Error: 1_564
			.saturating_add(Weight::from_parts(1_150_081, 0).saturating_mul(s.into()))
			// Standard Error: 1_564
			.saturating_add(Weight::from_parts(262_973, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(s.into())))
	}
	/// Storage: `Identity::IdentityOf` (r:1 w:0)
	/// Proof: `Identity::IdentityOf` (`max_values`: None, `max_size`: Some(7526), added: 10001, mode: `MaxEncodedLen`)
	/// Storage: `Identity::SuperOf` (r:1 w:1)
	/// Proof: `Identity::SuperOf` (`max_values`: None, `max_size`: Some(90), added: 2565, mode: `MaxEncodedLen`)
	/// Storage: `Identity::SubsOf` (r:1 w:1)
	/// Proof: `Identity::SubsOf` (`max_values`: None, `max_size`: Some(2046), added: 4521, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[0, 99]`.
	fn add_sub(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `535 + s * (23 ±0)`
		//  Estimated: `10991`
		// Minimum execution time: 23_000_000 picoseconds.
		Weight::from_parts(26_129_150, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 867
			.saturating_add(Weight::from_parts(12_583, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Identity::IdentityOf` (r:1 w:0)
	/// Proof: `Identity::IdentityOf` (`max_values`: None, `max_size`: Some(7526), added: 10001, mode: `MaxEncodedLen`)
	/// Storage: `Identity::SuperOf` (r:1 w:1)
	/// Proof: `Identity::SuperOf` (`max_values`: None, `max_size`: Some(90), added: 2565, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[1, 100]`.
	fn rename_sub(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `614 + s * (4 ±0)`
		//  Estimated: `10991`
		// Minimum execution time: 9_000_000 picoseconds.
		Weight::from_parts(10_275_327, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 954
			.saturating_add(Weight::from_parts(10_929, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Identity::IdentityOf` (r:1 w:0)
	/// Proof: `Identity::IdentityOf` (`max_values`: None, `max_size`: Some(7526), added: 10001, mode: `MaxEncodedLen`)
	/// Storage: `Identity::SuperOf` (r:1 w:1)
	/// Proof: `Identity::SuperOf` (`max_values`: None, `max_size`: Some(90), added: 2565, mode: `MaxEncodedLen`)
	/// Storage: `Identity::SubsOf` (r:1 w:1)
	/// Proof: `Identity::SubsOf` (`max_values`: None, `max_size`: Some(2046), added: 4521, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[1, 100]`.
	fn remove_sub(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `646 + s * (24 ±0)`
		//  Estimated: `10991`
		// Minimum execution time: 25_000_000 picoseconds.
		Weight::from_parts(27_088_961, 0)
			.saturating_add(Weight::from_parts(0, 10991))
			// Standard Error: 894
			.saturating_add(Weight::from_parts(16_534, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Identity::SuperOf` (r:1 w:1)
	/// Proof: `Identity::SuperOf` (`max_values`: None, `max_size`: Some(90), added: 2565, mode: `MaxEncodedLen`)
	/// Storage: `Identity::SubsOf` (r:1 w:1)
	/// Proof: `Identity::SubsOf` (`max_values`: None, `max_size`: Some(2046), added: 4521, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:0)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(116), added: 2591, mode: `MaxEncodedLen`)
	/// The range of component `s` is `[0, 99]`.
	fn quit_sub(s: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `658 + s * (24 ±0)`
		//  Estimated: `5511`
		// Minimum execution time: 17_000_000 picoseconds.
		Weight::from_parts(18_820_598, 0)
			.saturating_add(Weight::from_parts(0, 5511))
			// Standard Error: 807
			.saturating_add(Weight::from_parts(15_940, 0).saturating_mul(s.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
}
