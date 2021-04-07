#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_im_online::WeightInfo for WeightInfo<T> {
	fn validate_unsigned_and_then_heartbeat(k: u32, e: u32) -> Weight {
		(107_274_000 as Weight)
			.saturating_add((218_000 as Weight).saturating_mul(k as Weight))
			.saturating_add((482_000 as Weight).saturating_mul(e as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
