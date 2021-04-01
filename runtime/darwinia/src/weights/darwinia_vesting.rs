#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> darwinia_vesting::WeightInfo for WeightInfo<T> {
	fn vest_locked(l: u32) -> Weight {
		(53_484_000 as Weight)
			.saturating_add((134_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn vest_unlocked(l: u32) -> Weight {
		(57_798_000 as Weight)
			.saturating_add((109_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn vest_other_locked(l: u32) -> Weight {
		(53_265_000 as Weight)
			.saturating_add((130_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn vest_other_unlocked(l: u32) -> Weight {
		(57_589_000 as Weight)
			.saturating_add((107_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn vested_transfer(l: u32) -> Weight {
		(118_226_000 as Weight)
			.saturating_add((164_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn force_vested_transfer(l: u32) -> Weight {
		(117_385_000 as Weight)
			.saturating_add((166_000 as Weight).saturating_mul(l as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
}
