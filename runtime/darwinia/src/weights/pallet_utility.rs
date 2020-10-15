//! Weights for pallet_utility
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0
//! DATE: 2020-09-28, STEPS: [50], REPEAT: 20, LOW RANGE: [], HIGH RANGE: []

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Trait> pallet_utility::WeightInfo for WeightInfo<T> {
	fn batch(c: u32) -> Weight {
		(18_589_000 as Weight).saturating_add((1_734_000 as Weight).saturating_mul(c as Weight))
	}
	// WARNING! Some components were not used: ["u"]
	fn as_derivative() -> Weight {
		(5_611_000 as Weight)
	}
}
