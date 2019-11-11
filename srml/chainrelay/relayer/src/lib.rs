//!  prototype module for bridging in ethereum poa blockcahin

#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use support::{decl_event, decl_module, decl_storage};

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Bridge {
		pub EOSRelayer get(eos_relayer): map T::AccountId => Relayer;
		pub EthereumRelayer get(ethereum_relayer): map T::AccountId => Relayer;
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

#[derive(Default, Decode, Encode)]
pub struct Relayer {
	contribution: i32,
}
