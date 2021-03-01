#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Trait> pallet_collective::WeightInfo for WeightInfo<T> {
	fn set_members(m: u32, n: u32, p: u32) -> Weight {
		(0 as Weight)
			.saturating_add((20_744_000 as Weight).saturating_mul(m as Weight))
			.saturating_add((71_000 as Weight).saturating_mul(n as Weight))
			.saturating_add((27_804_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(p as Weight)))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(p as Weight)))
	}
	fn execute(b: u32, m: u32) -> Weight {
		(29_512_000 as Weight)
			.saturating_add((4_000 as Weight).saturating_mul(b as Weight))
			.saturating_add((112_000 as Weight).saturating_mul(m as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
	}
	fn propose_execute(b: u32, m: u32) -> Weight {
		(36_554_000 as Weight)
			.saturating_add((4_000 as Weight).saturating_mul(b as Weight))
			.saturating_add((220_000 as Weight).saturating_mul(m as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
	}
	fn propose_proposed(b: u32, m: u32, p: u32) -> Weight {
		(60_010_000 as Weight)
			.saturating_add((5_000 as Weight).saturating_mul(b as Weight))
			.saturating_add((115_000 as Weight).saturating_mul(m as Weight))
			.saturating_add((565_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn vote(m: u32) -> Weight {
		(43_161_000 as Weight)
			.saturating_add((277_000 as Weight).saturating_mul(m as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn close_early_disapproved(m: u32, p: u32) -> Weight {
		(57_179_000 as Weight)
			.saturating_add((215_000 as Weight).saturating_mul(m as Weight))
			.saturating_add((527_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn close_early_approved(b: u32, m: u32, p: u32) -> Weight {
		(82_532_000 as Weight)
			.saturating_add((3_000 as Weight).saturating_mul(b as Weight))
			.saturating_add((220_000 as Weight).saturating_mul(m as Weight))
			.saturating_add((532_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn close_disapproved(m: u32, p: u32) -> Weight {
		(63_597_000 as Weight)
			.saturating_add((219_000 as Weight).saturating_mul(m as Weight))
			.saturating_add((526_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn close_approved(b: u32, m: u32, p: u32) -> Weight {
		(88_919_000 as Weight)
			.saturating_add((3_000 as Weight).saturating_mul(b as Weight))
			.saturating_add((220_000 as Weight).saturating_mul(m as Weight))
			.saturating_add((532_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn disapprove_proposal(p: u32) -> Weight {
		(33_674_000 as Weight)
			.saturating_add((527_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
}
