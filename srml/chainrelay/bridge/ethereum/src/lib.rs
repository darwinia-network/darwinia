//!  prototype module for bridging in ethereum poa blockcahin

#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

// use blake2::Blake2b;
//use codec::{Decode, Encode};
use rstd::vec::Vec;
use support::{
	decl_event, decl_module, decl_storage,
	dispatch::Result,
	traits::{Currency, LockableCurrency},
};
use system::ensure_signed;

use darwinia_support::types::TimeStamp;
//use merkle_mountain_range::{MerkleMountainRange, Hash};

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Ring: LockableCurrency<Self::AccountId, Moment = TimeStamp>;
}

// config() require `serde = { version = "1.0.101", optional = true }`
// tracking issue: https://github.com/rust-lang/rust/issues/27812
decl_storage! {
	trait Store for Module<T: Trait> as Bridge {
		pub DepositPool get(deposit_pool) config(): RingBalanceOf<T>;
		pub DepositValue get(deposit_value): RingBalanceOf<T>;

		// store Vec<Header> or MPT<Header>?
		pub VerifiedHeader get(verified_header): Vec<Header>;
		pub UnverifiedHeader get(unverified_header): map PrevHash => Vec<Header>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call
	where
		origin: T::Origin
	{
		pub fn submit_header(origin, header: Header) {
			let _relayer = ensure_signed(origin)?;
			let _ = Self::verify(&header)?;

			// if header confirmed then return
			// if header in unverified header then challenge
		 }

		// `Darwinia lock` corresponds to `TargetChain redeem`
		pub fn lock(origin) {
			let _locker = ensure_signed(origin)?;
		}

		// `Darwinia redeem` corresponds to `TargetChain lock`
		pub fn redeem(origin, _header: Header) {
			let _redeemer = ensure_signed(origin)?;
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
	pub fn adjust_deposit_value() {
		unimplemented!()
	}

	/// 1. if exists?
	/// 2. verify (difficulty + prev_hash + nonce)
	/// 3. challenge
	fn verify(_: &Header) -> Result {
		unimplemented!()
	}

	fn _punish(_who: &T::AccountId) -> Result {
		unimplemented!()
	}

	fn _release(_dest: &T::AccountId, _value: RingBalanceOf<T>) -> Result {
		unimplemented!()
	}
}

type RingBalanceOf<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::Balance;
// TODO: type
type Header = ();
type PrevHash = ();
// FIXME: currently, use SPV instead
// pub type MMR = MerkleMountainRange<Blake2b>;
