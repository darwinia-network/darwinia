#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Trait> pallet_collective::WeightInfo for WeightInfo<T> {
	fn set_members(m: u32, n: u32, p: u32) -> Weight {
		(0 as Weight)
			.saturating_add((20_942_000 as Weight).saturating_mul(m as Weight))
			.saturating_add((124_000 as Weight).saturating_mul(n as Weight))
			.saturating_add((27_863_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(p as Weight)))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(p as Weight)))
	}
	fn execute(b: u32, m: u32) -> Weight {
		(29_288_000 as Weight)
			.saturating_add((4_000 as Weight).saturating_mul(b as Weight))
			.saturating_add((107_000 as Weight).saturating_mul(m as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
	}
	fn propose_execute(b: u32, m: u32) -> Weight {
		(36_744_000 as Weight)
			.saturating_add((3_000 as Weight).saturating_mul(b as Weight))
			.saturating_add((215_000 as Weight).saturating_mul(m as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
	}
	fn propose_proposed(b: u32, m: u32, p: u32) -> Weight {
		(59_415_000 as Weight)
			.saturating_add((5_000 as Weight).saturating_mul(b as Weight))
			.saturating_add((119_000 as Weight).saturating_mul(m as Weight))
			.saturating_add((573_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn vote(m: u32) -> Weight {
		(42_965_000 as Weight)
			.saturating_add((277_000 as Weight).saturating_mul(m as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn close_early_disapproved(m: u32, p: u32) -> Weight {
		(56_592_000 as Weight)
			.saturating_add((219_000 as Weight).saturating_mul(m as Weight))
			.saturating_add((539_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn close_early_approved(b: u32, m: u32, p: u32) -> Weight {
		(82_480_000 as Weight)
			.saturating_add((4_000 as Weight).saturating_mul(b as Weight))
			.saturating_add((220_000 as Weight).saturating_mul(m as Weight))
			.saturating_add((539_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn close_disapproved(m: u32, p: u32) -> Weight {
		(63_358_000 as Weight)
			.saturating_add((220_000 as Weight).saturating_mul(m as Weight))
			.saturating_add((543_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn close_approved(b: u32, m: u32, p: u32) -> Weight {
		(88_301_000 as Weight)
			.saturating_add((3_000 as Weight).saturating_mul(b as Weight))
			.saturating_add((223_000 as Weight).saturating_mul(m as Weight))
			.saturating_add((554_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn disapprove_proposal(p: u32) -> Weight {
		(33_338_000 as Weight)
			.saturating_add((543_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
}
