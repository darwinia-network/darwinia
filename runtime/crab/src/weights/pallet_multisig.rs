#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Trait> pallet_multisig::WeightInfo for WeightInfo<T> {
	fn as_multi_threshold_1(z: u32) -> Weight {
		(12_168_000 as Weight).saturating_add((1_000 as Weight).saturating_mul(z as Weight))
	}
	fn as_multi_create(s: u32, z: u32) -> Weight {
		(66_343_000 as Weight)
			.saturating_add((87_000 as Weight).saturating_mul(s as Weight))
			.saturating_add((1_000 as Weight).saturating_mul(z as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn as_multi_create_store(s: u32, z: u32) -> Weight {
		(73_923_000 as Weight)
			.saturating_add((91_000 as Weight).saturating_mul(s as Weight))
			.saturating_add((3_000 as Weight).saturating_mul(z as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn as_multi_approve(s: u32, z: u32) -> Weight {
		(40_235_000 as Weight)
			.saturating_add((109_000 as Weight).saturating_mul(s as Weight))
			.saturating_add((1_000 as Weight).saturating_mul(z as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn as_multi_approve_store(s: u32, z: u32) -> Weight {
		(71_596_000 as Weight)
			.saturating_add((122_000 as Weight).saturating_mul(s as Weight))
			.saturating_add((3_000 as Weight).saturating_mul(z as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn as_multi_complete(s: u32, z: u32) -> Weight {
		(82_197_000 as Weight)
			.saturating_add((248_000 as Weight).saturating_mul(s as Weight))
			.saturating_add((5_000 as Weight).saturating_mul(z as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn approve_as_multi_create(s: u32) -> Weight {
		(65_473_000 as Weight)
			.saturating_add((89_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn approve_as_multi_approve(s: u32) -> Weight {
		(38_965_000 as Weight)
			.saturating_add((108_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn approve_as_multi_complete(s: u32) -> Weight {
		(153_044_000 as Weight)
			.saturating_add((251_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn cancel_as_multi(s: u32) -> Weight {
		(107_171_000 as Weight)
			.saturating_add((89_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
}
