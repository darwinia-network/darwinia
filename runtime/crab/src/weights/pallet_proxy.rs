#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Trait> pallet_proxy::WeightInfo for WeightInfo<T> {
	fn proxy(p: u32) -> Weight {
		(30_797_000 as Weight)
			.saturating_add((182_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
	}
	fn proxy_announced(a: u32, p: u32) -> Weight {
		(65_387_000 as Weight)
			.saturating_add((849_000 as Weight).saturating_mul(a as Weight))
			.saturating_add((183_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn remove_announcement(a: u32, p: u32) -> Weight {
		(41_839_000 as Weight)
			.saturating_add((832_000 as Weight).saturating_mul(a as Weight))
			.saturating_add((7_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn reject_announcement(a: u32, p: u32) -> Weight {
		(41_701_000 as Weight)
			.saturating_add((835_000 as Weight).saturating_mul(a as Weight))
			.saturating_add((11_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn announce(a: u32, p: u32) -> Weight {
		(66_233_000 as Weight)
			.saturating_add((741_000 as Weight).saturating_mul(a as Weight))
			.saturating_add((194_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn add_proxy(p: u32) -> Weight {
		(45_088_000 as Weight)
			.saturating_add((194_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn remove_proxy(p: u32) -> Weight {
		(40_241_000 as Weight)
			.saturating_add((226_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn remove_proxies(p: u32) -> Weight {
		(38_268_000 as Weight)
			.saturating_add((183_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn anonymous(p: u32) -> Weight {
		(63_718_000 as Weight)
			.saturating_add((29_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn kill_anonymous(p: u32) -> Weight {
		(40_839_000 as Weight)
			.saturating_add((189_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
