//! Weights for pallet_timestamp
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0
//! DATE: 2020-09-28, STEPS: [50], REPEAT: 20, LOW RANGE: [], HIGH RANGE: []

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Trait> pallet_timestamp::WeightInfo for WeightInfo<T> {
	// WARNING! Some components were not used: ["t"]
	fn set() -> Weight {
		(11_029_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// WARNING! Some components were not used: ["t"]
	fn on_finalize() -> Weight {
		(6_128_000 as Weight)
	}
}
