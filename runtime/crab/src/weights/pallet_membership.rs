#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for pallet_membership.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_membership::WeightInfo for WeightInfo<T> {
	fn add_member(m: u32) -> Weight {
		(22_578_000 as Weight)
			// Standard Error: 3_000
			.saturating_add((240_000 as Weight).saturating_mul(m as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn remove_member(m: u32) -> Weight {
		(27_590_000 as Weight)
			// Standard Error: 0
			.saturating_add((194_000 as Weight).saturating_mul(m as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn swap_member(m: u32) -> Weight {
		(27_451_000 as Weight)
			// Standard Error: 0
			.saturating_add((212_000 as Weight).saturating_mul(m as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn reset_member(m: u32) -> Weight {
		(27_655_000 as Weight)
			// Standard Error: 0
			.saturating_add((428_000 as Weight).saturating_mul(m as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn change_key(m: u32) -> Weight {
		(29_256_000 as Weight)
			// Standard Error: 0
			.saturating_add((206_000 as Weight).saturating_mul(m as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn set_prime(m: u32) -> Weight {
		(7_334_000 as Weight)
			// Standard Error: 0
			.saturating_add((113_000 as Weight).saturating_mul(m as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn clear_prime(m: u32) -> Weight {
		(2_741_000 as Weight)
			// Standard Error: 0
			.saturating_add((1_000 as Weight).saturating_mul(m as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
}
