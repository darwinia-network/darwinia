#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> darwinia_democracy::WeightInfo for WeightInfo<T> {
	fn propose() -> Weight {
		(73_769_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn second(s: u32) -> Weight {
		(48_621_000 as Weight)
			.saturating_add((191_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn vote_new(r: u32) -> Weight {
		(58_568_000 as Weight)
			.saturating_add((224_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn vote_existing(r: u32) -> Weight {
		(58_374_000 as Weight)
			.saturating_add((229_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn emergency_cancel() -> Weight {
		(35_851_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn blacklist(p: u32) -> Weight {
		(117_822_000 as Weight)
			.saturating_add((802_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	fn external_propose(v: u32) -> Weight {
		(17_593_000 as Weight)
			.saturating_add((110_000 as Weight).saturating_mul(v as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn external_propose_majority() -> Weight {
		(4_225_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn external_propose_default() -> Weight {
		(4_148_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn fast_track() -> Weight {
		(36_860_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn veto_external(v: u32) -> Weight {
		(38_043_000 as Weight)
			.saturating_add((178_000 as Weight).saturating_mul(v as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn cancel_proposal(p: u32) -> Weight {
		(81_567_000 as Weight)
			.saturating_add((876_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn cancel_referendum() -> Weight {
		(21_906_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn cancel_queued(r: u32) -> Weight {
		(41_109_000 as Weight)
			.saturating_add((3_388_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn on_initialize_base(r: u32) -> Weight {
		(13_877_000 as Weight)
			.saturating_add((6_543_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(r as Weight)))
	}
	fn delegate(r: u32) -> Weight {
		(76_902_000 as Weight)
			.saturating_add((9_605_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(r as Weight)))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(r as Weight)))
	}
	fn undelegate(r: u32) -> Weight {
		(39_807_000 as Weight)
			.saturating_add((9_502_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(r as Weight)))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(r as Weight)))
	}
	fn clear_public_proposals() -> Weight {
		(3_443_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn note_preimage(b: u32) -> Weight {
		(55_525_000 as Weight)
			.saturating_add((4_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn note_imminent_preimage(b: u32) -> Weight {
		(37_807_000 as Weight)
			.saturating_add((3_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn reap_preimage(b: u32) -> Weight {
		(51_485_000 as Weight)
			.saturating_add((3_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn unlock_remove(r: u32) -> Weight {
		(49_585_000 as Weight)
			.saturating_add((37_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn unlock_set(r: u32) -> Weight {
		(44_824_000 as Weight)
			.saturating_add((220_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn remove_vote(r: u32) -> Weight {
		(27_128_000 as Weight)
			.saturating_add((213_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn remove_other_vote(r: u32) -> Weight {
		(27_306_000 as Weight)
			.saturating_add((214_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
}
