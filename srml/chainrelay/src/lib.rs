//!  prototype module for bridging in ethereum poa blockcahin

#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

use rstd::vec::Vec;

// use blake2::Blake2b;
use codec::{Decode, Encode};
use support::{decl_event, decl_module, decl_storage, dispatch::Result};
use system::ensure_signed;

use merkle_mountain_range::Hash;
//use merkle_mountain_range::MerkleMountainRange;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Bridge {
		pub ChainHeaders get(chain_headers): map Chain => Vec<Header>;
		pub RelayerContribution get(relayer_contribution): map T::AccountId => u32;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call
	where
		origin: T::Origin
	{
		pub fn submit_header(origin, chain: Chain, header: Header) {
			let relayer = ensure_signed(origin)?;
			let _ = Self::verify()?;

			<RelayerContribution<T>>::mutate(relayer, |r_c| {
				*r_c += 1;
			});
			ChainHeaders::mutate(chain, |c_h| c_h.push(header));
		}
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

impl<T: Trait> Module<T> {
	fn verify() -> Result {
		/// 1. if exists?
		/// 2. verify (difficulty + prev_hash + nonce)
		/// 3. challenge
		unimplemented!()
	}
}

// pub type MMR = MerkleMountainRange<Blake2b>;

#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode)]
pub enum Chain {
	Ethereum,
}

#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode)]
pub struct Header {
	difficulty: u32,
	nonce: u32,
	prev_hash: Hash,
	merkle_root: Hash,
}
