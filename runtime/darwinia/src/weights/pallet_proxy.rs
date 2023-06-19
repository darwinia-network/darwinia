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

//! Autogenerated weights for `pallet_proxy`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-06-19, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `inv.cafe`, CPU: `13th Gen Intel(R) Core(TM) i9-13900K`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("darwinia-local"), DB CACHE: 1024

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
// darwinia-local
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

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_proxy`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_proxy::WeightInfo for WeightInfo<T> {
	/// Storage: Proxy Proxies (r:1 w:0)
	/// Proof: Proxy Proxies (max_values: None, max_size: Some(845), added: 3320, mode: MaxEncodedLen)
	/// The range of component `p` is `[1, 31]`.
	fn proxy(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `149 + p * (25 ±0)`
		//  Estimated: `4310`
		// Minimum execution time: 10_116_000 picoseconds.
		Weight::from_parts(10_722_865, 0)
			.saturating_add(Weight::from_parts(0, 4310))
			// Standard Error: 9_714
			.saturating_add(Weight::from_parts(53_629, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	/// Storage: Proxy Proxies (r:1 w:0)
	/// Proof: Proxy Proxies (max_values: None, max_size: Some(845), added: 3320, mode: MaxEncodedLen)
	/// Storage: Proxy Announcements (r:1 w:1)
	/// Proof: Proxy Announcements (max_values: None, max_size: Some(1837), added: 4312, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	/// The range of component `a` is `[0, 31]`.
	/// The range of component `p` is `[1, 31]`.
	fn proxy_announced(a: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `401 + a * (56 ±0) + p * (25 ±0)`
		//  Estimated: `13193`
		// Minimum execution time: 23_072_000 picoseconds.
		Weight::from_parts(23_404_951, 0)
			.saturating_add(Weight::from_parts(0, 13193))
			// Standard Error: 7_296
			.saturating_add(Weight::from_parts(180_904, 0).saturating_mul(a.into()))
			// Standard Error: 7_539
			.saturating_add(Weight::from_parts(72_102, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Proxy Announcements (r:1 w:1)
	/// Proof: Proxy Announcements (max_values: None, max_size: Some(1837), added: 4312, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	/// The range of component `a` is `[0, 31]`.
	/// The range of component `p` is `[1, 31]`.
	fn remove_announcement(a: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `329 + a * (56 ±0)`
		//  Estimated: `8883`
		// Minimum execution time: 14_573_000 picoseconds.
		Weight::from_parts(15_449_865, 0)
			.saturating_add(Weight::from_parts(0, 8883))
			// Standard Error: 3_575
			.saturating_add(Weight::from_parts(148_540, 0).saturating_mul(a.into()))
			// Standard Error: 3_694
			.saturating_add(Weight::from_parts(17_473, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Proxy Announcements (r:1 w:1)
	/// Proof: Proxy Announcements (max_values: None, max_size: Some(1837), added: 4312, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	/// The range of component `a` is `[0, 31]`.
	/// The range of component `p` is `[1, 31]`.
	fn reject_announcement(a: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `329 + a * (56 ±0)`
		//  Estimated: `8883`
		// Minimum execution time: 14_207_000 picoseconds.
		Weight::from_parts(15_464_710, 0)
			.saturating_add(Weight::from_parts(0, 8883))
			// Standard Error: 3_427
			.saturating_add(Weight::from_parts(146_699, 0).saturating_mul(a.into()))
			// Standard Error: 3_541
			.saturating_add(Weight::from_parts(19_046, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Proxy Proxies (r:1 w:0)
	/// Proof: Proxy Proxies (max_values: None, max_size: Some(845), added: 3320, mode: MaxEncodedLen)
	/// Storage: Proxy Announcements (r:1 w:1)
	/// Proof: Proxy Announcements (max_values: None, max_size: Some(1837), added: 4312, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(116), added: 2591, mode: MaxEncodedLen)
	/// The range of component `a` is `[0, 31]`.
	/// The range of component `p` is `[1, 31]`.
	fn announce(a: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `345 + a * (56 ±0) + p * (25 ±0)`
		//  Estimated: `13193`
		// Minimum execution time: 20_741_000 picoseconds.
		Weight::from_parts(22_257_680, 0)
			.saturating_add(Weight::from_parts(0, 13193))
			// Standard Error: 4_384
			.saturating_add(Weight::from_parts(159_481, 0).saturating_mul(a.into()))
			// Standard Error: 4_530
			.saturating_add(Weight::from_parts(23_322, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Proxy Proxies (r:1 w:1)
	/// Proof: Proxy Proxies (max_values: None, max_size: Some(845), added: 3320, mode: MaxEncodedLen)
	/// The range of component `p` is `[1, 31]`.
	fn add_proxy(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `149 + p * (25 ±0)`
		//  Estimated: `4310`
		// Minimum execution time: 15_802_000 picoseconds.
		Weight::from_parts(17_002_956, 0)
			.saturating_add(Weight::from_parts(0, 4310))
			// Standard Error: 10_213
			.saturating_add(Weight::from_parts(58_635, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Proxy Proxies (r:1 w:1)
	/// Proof: Proxy Proxies (max_values: None, max_size: Some(845), added: 3320, mode: MaxEncodedLen)
	/// The range of component `p` is `[1, 31]`.
	fn remove_proxy(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `149 + p * (25 ±0)`
		//  Estimated: `4310`
		// Minimum execution time: 15_581_000 picoseconds.
		Weight::from_parts(16_801_078, 0)
			.saturating_add(Weight::from_parts(0, 4310))
			// Standard Error: 9_041
			.saturating_add(Weight::from_parts(75_458, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Proxy Proxies (r:1 w:1)
	/// Proof: Proxy Proxies (max_values: None, max_size: Some(845), added: 3320, mode: MaxEncodedLen)
	/// The range of component `p` is `[1, 31]`.
	fn remove_proxies(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `149 + p * (25 ±0)`
		//  Estimated: `4310`
		// Minimum execution time: 12_444_000 picoseconds.
		Weight::from_parts(13_708_257, 0)
			.saturating_add(Weight::from_parts(0, 4310))
			// Standard Error: 11_974
			.saturating_add(Weight::from_parts(37_805, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Proxy Proxies (r:1 w:1)
	/// Proof: Proxy Proxies (max_values: None, max_size: Some(845), added: 3320, mode: MaxEncodedLen)
	/// The range of component `p` is `[1, 31]`.
	fn create_pure(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `161`
		//  Estimated: `4310`
		// Minimum execution time: 17_328_000 picoseconds.
		Weight::from_parts(18_345_399, 0)
			.saturating_add(Weight::from_parts(0, 4310))
			// Standard Error: 14_465
			.saturating_add(Weight::from_parts(51_736, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Proxy Proxies (r:1 w:1)
	/// Proof: Proxy Proxies (max_values: None, max_size: Some(845), added: 3320, mode: MaxEncodedLen)
	/// The range of component `p` is `[0, 30]`.
	fn kill_pure(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `174 + p * (25 ±0)`
		//  Estimated: `4310`
		// Minimum execution time: 12_995_000 picoseconds.
		Weight::from_parts(13_611_951, 0)
			.saturating_add(Weight::from_parts(0, 4310))
			// Standard Error: 10_790
			.saturating_add(Weight::from_parts(68_920, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
