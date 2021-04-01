#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> darwinia_elections_phragmen::WeightInfo for WeightInfo<T> {
	fn vote_equal(v: u32) -> Weight {
		(54_923_000 as Weight)
			.saturating_add((324_000 as Weight).saturating_mul(v as Weight))
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn vote_more(v: u32) -> Weight {
		(83_389_000 as Weight)
			.saturating_add((341_000 as Weight).saturating_mul(v as Weight))
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn vote_less(v: u32) -> Weight {
		(78_865_000 as Weight)
			.saturating_add((343_000 as Weight).saturating_mul(v as Weight))
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn remove_voter() -> Weight {
		(72_370_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn submit_candidacy(c: u32) -> Weight {
		(68_455_000 as Weight)
			.saturating_add((370_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn renounce_candidacy_candidate(c: u32) -> Weight {
		(54_009_000 as Weight)
			.saturating_add((200_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn renounce_candidacy_members() -> Weight {
		(84_797_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn renounce_candidacy_runners_up() -> Weight {
		(59_095_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn remove_member_with_replacement() -> Weight {
		(132_820_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	fn remove_member_wrong_refund() -> Weight {
		(8_551_000 as Weight).saturating_add(T::DbWeight::get().reads(1 as Weight))
	}
	fn clean_defunct_voters(v: u32, _d: u32) -> Weight {
		(0 as Weight)
			.saturating_add((151_754_000 as Weight).saturating_mul(v as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().reads((3 as Weight).saturating_mul(v as Weight)))
			.saturating_add(T::DbWeight::get().writes((3 as Weight).saturating_mul(v as Weight)))
	}
	fn election_phragmen(c: u32, v: u32, e: u32) -> Weight {
		(0 as Weight)
			.saturating_add((134_602_000 as Weight).saturating_mul(c as Weight))
			.saturating_add((111_037_000 as Weight).saturating_mul(v as Weight))
			.saturating_add((7_150_000 as Weight).saturating_mul(e as Weight))
			.saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(c as Weight)))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(v as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
	}
}
