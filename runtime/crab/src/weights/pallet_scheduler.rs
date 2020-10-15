//! Weights for pallet_scheduler
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0
//! DATE: 2020-09-28, STEPS: [50], REPEAT: 20, LOW RANGE: [], HIGH RANGE: []

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Trait> pallet_scheduler::WeightInfo for WeightInfo<T> {
	fn schedule(s: u32) -> Weight {
		(32_493_000 as Weight)
			.saturating_add((49_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn cancel(s: u32) -> Weight {
		(29_719_000 as Weight)
			.saturating_add((3_071_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn schedule_named(s: u32) -> Weight {
		(41_811_000 as Weight)
			.saturating_add((61_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn cancel_named(s: u32) -> Weight {
		(33_725_000 as Weight)
			.saturating_add((3_082_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
}
