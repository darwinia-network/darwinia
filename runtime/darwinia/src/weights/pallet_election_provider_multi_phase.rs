#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for pallet_election_provider_multi_phase.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_election_provider_multi_phase::WeightInfo for WeightInfo<T> {
	fn on_initialize_nothing() -> Weight {
		(19_802_000 as Weight).saturating_add(T::DbWeight::get().reads(7 as Weight))
	}
	fn on_initialize_open_signed() -> Weight {
		(100_008_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn on_initialize_open_unsigned_with_snapshot() -> Weight {
		(99_255_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn on_initialize_open_unsigned_without_snapshot() -> Weight {
		(18_383_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn elect_queued() -> Weight {
		(7_631_033_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	fn submit_unsigned(v: u32, _t: u32, a: u32, d: u32) -> Weight {
		(0 as Weight)
			// Standard Error: 19_000
			.saturating_add((4_133_000 as Weight).saturating_mul(v as Weight))
			// Standard Error: 19_000
			.saturating_add((12_776_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 99_000
			.saturating_add((2_879_000 as Weight).saturating_mul(d as Weight))
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn feasibility_check(v: u32, t: u32, a: u32, d: u32) -> Weight {
		(0 as Weight)
			// Standard Error: 11_000
			.saturating_add((4_286_000 as Weight).saturating_mul(v as Weight))
			// Standard Error: 39_000
			.saturating_add((468_000 as Weight).saturating_mul(t as Weight))
			// Standard Error: 11_000
			.saturating_add((9_291_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 58_000
			.saturating_add((3_405_000 as Weight).saturating_mul(d as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
	}
}
