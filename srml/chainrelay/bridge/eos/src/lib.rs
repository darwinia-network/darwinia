//!  prototype module for bridging in ethereum poa blockcahin

#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

//use codec::{Decode, Encode};
use support::{
	decl_event,
	decl_module,
	decl_storage,
	//	dispatch::Result,
	traits::{Currency, LockableCurrency},
};

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Ring: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Bridge {

	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call
	where
		origin: T::Origin
	{

	}
}

decl_event! {
	pub enum Event<T>
	where
		<T as system::Trait>::AccountId
	{
		TODO(AccountId),
	}
}

impl<T: Trait> Module<T> {}

type RingBalanceOf<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::Balance;
// FIXME: currently, use SPV instead
// pub type MMR = MerkleMountainRange<Blake2b>;
