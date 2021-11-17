#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for pallet_election_provider_multi_phase.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_election_provider_multi_phase::WeightInfo for WeightInfo<T> {
	fn on_initialize_nothing() -> Weight {
		(23_244_000 as Weight).saturating_add(T::DbWeight::get().reads(8 as Weight))
	}
	fn on_initialize_open_signed() -> Weight {
		(82_453_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(10 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn on_initialize_open_unsigned_with_snapshot() -> Weight {
		(81_883_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(10 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn on_initialize_open_unsigned_without_snapshot() -> Weight {
		(17_601_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn elect_queued() -> Weight {
		(5_408_539_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	fn submit_unsigned(v: u32, t: u32, a: u32, d: u32) -> Weight {
		(0 as Weight)
			// Standard Error: 15_000
			.saturating_add((3_352_000 as Weight).saturating_mul(v as Weight))
			// Standard Error: 52_000
			.saturating_add((150_000 as Weight).saturating_mul(t as Weight))
			// Standard Error: 15_000
			.saturating_add((10_531_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 78_000
			.saturating_add((3_302_000 as Weight).saturating_mul(d as Weight))
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn feasibility_check(v: u32, t: u32, a: u32, d: u32) -> Weight {
		(0 as Weight)
			// Standard Error: 10_000
			.saturating_add((3_365_000 as Weight).saturating_mul(v as Weight))
			// Standard Error: 34_000
			.saturating_add((295_000 as Weight).saturating_mul(t as Weight))
			// Standard Error: 10_000
			.saturating_add((8_438_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 52_000
			.saturating_add((3_606_000 as Weight).saturating_mul(d as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
	}
	fn finalize_signed_phase_accept_solution() -> Weight {
		(47_783_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn finalize_signed_phase_reject_solution() -> Weight {
		(21_277_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn submit(c: u32) -> Weight {
		(78_972_000 as Weight)
			// Standard Error: 16_000
			.saturating_add((308_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
}
