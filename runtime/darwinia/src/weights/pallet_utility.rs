#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Trait> pallet_utility::WeightInfo for WeightInfo<T> {
	fn batch(c: u32) -> Weight {
		(18_624_000 as Weight).saturating_add((1_986_000 as Weight).saturating_mul(c as Weight))
	}
	fn as_derivative() -> Weight {
		(5_576_000 as Weight)
	}
	fn batch_all(c: u32) -> Weight {
		(19_708_000 as Weight).saturating_add((1_988_000 as Weight).saturating_mul(c as Weight))
	}
}
