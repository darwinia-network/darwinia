#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for darwinia_vesting.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> darwinia_vesting::WeightInfo for WeightInfo<T> {
	fn vest_locked(l: u32) -> Weight {
		(41_841_000 as Weight)
			// Standard Error: 13_000
			.saturating_add((202_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn vest_unlocked(l: u32) -> Weight {
		(45_276_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((133_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn vest_other_locked(l: u32) -> Weight {
		(41_217_000 as Weight)
			// Standard Error: 12_000
			.saturating_add((217_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn vest_other_unlocked(l: u32) -> Weight {
		(44_521_000 as Weight)
			// Standard Error: 5_000
			.saturating_add((163_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn vested_transfer(l: u32) -> Weight {
		(101_636_000 as Weight)
			// Standard Error: 12_000
			.saturating_add((105_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn force_vested_transfer(l: u32) -> Weight {
		(99_417_000 as Weight)
			// Standard Error: 12_000
			.saturating_add((137_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
}
