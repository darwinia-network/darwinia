#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_election_provider_multi_phase`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_election_provider_multi_phase::WeightInfo for WeightInfo<T> {
	fn on_initialize_nothing() -> Weight {
		(22_984_000 as Weight).saturating_add(T::DbWeight::get().reads(8 as Weight))
	}
	fn on_initialize_open_signed() -> Weight {
		(83_667_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(10 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn on_initialize_open_unsigned_with_snapshot() -> Weight {
		(83_403_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(10 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn on_initialize_open_unsigned_without_snapshot() -> Weight {
		(18_070_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn elect_queued(_v: u32, _t: u32, _a: u32, _d: u32) -> Weight {
		(8_641_847_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	fn submit(c: u32) -> Weight {
		(84_430_000 as Weight)
			// Standard Error: 146_000
			.saturating_add((2_758_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn submit_unsigned(v: u32, t: u32, a: u32, d: u32) -> Weight {
		(0 as Weight)
			// Standard Error: 13_000
			.saturating_add((4_805_000 as Weight).saturating_mul(v as Weight))
			// Standard Error: 44_000
			.saturating_add((305_000 as Weight).saturating_mul(t as Weight))
			// Standard Error: 13_000
			.saturating_add((16_090_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 67_000
			.saturating_add((5_619_000 as Weight).saturating_mul(d as Weight))
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn feasibility_check(v: u32, t: u32, a: u32, d: u32) -> Weight {
		(0 as Weight)
			// Standard Error: 8_000
			.saturating_add((4_729_000 as Weight).saturating_mul(v as Weight))
			// Standard Error: 29_000
			.saturating_add((124_000 as Weight).saturating_mul(t as Weight))
			// Standard Error: 8_000
			.saturating_add((13_511_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 44_000
			.saturating_add((4_469_000 as Weight).saturating_mul(d as Weight))
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
}
