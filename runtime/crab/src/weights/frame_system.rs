//! Weights for frame_system
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0
//! DATE: 2020-09-28, STEPS: [50], REPEAT: 20, LOW RANGE: [], HIGH RANGE: []

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Trait> frame_system::WeightInfo for WeightInfo<T> {
	// WARNING! Some components were not used: ["b"]
	fn remark() -> Weight {
		(1_861_000 as Weight)
	}
	fn set_heap_pages() -> Weight {
		(2_431_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn set_changes_trie_config() -> Weight {
		(9_680_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn set_storage(i: u32) -> Weight {
		(0 as Weight)
			.saturating_add((793_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(i as Weight)))
	}
	fn kill_storage(i: u32) -> Weight {
		(0 as Weight)
			.saturating_add((552_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(i as Weight)))
	}
	fn kill_prefix(p: u32) -> Weight {
		(0 as Weight)
			.saturating_add((865_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(p as Weight)))
	}
	fn suicide() -> Weight {
		(35_363_000 as Weight)
	}
}
