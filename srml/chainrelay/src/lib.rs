//!  prototype module for bridging in ethereum poa blockcahin

#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod bridge;

// use blake2::Blake2b;
use codec::{Decode, Encode};
use support::{
	decl_event, decl_module, decl_storage,
	dispatch::Result,
	traits::{Currency, LockableCurrency},
};
use system::ensure_signed;

//use bridge::{Bridge, Relayer};
use bridge::{EOSBridge as EOSBridgeT, EOSRelayer as EOSRelayerT};
use bridge::{EthereumBridge as EthereumBridgeT, EthereumRelayer as EthereumRelayerT};
//use merkle_mountain_range::{MerkleMountainRange, Hash};
use crate::bridge::Bridge;
use TargetChain::*;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Ring: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
}

// config() require `serde = { version = "1.0.101", optional = true }`
// tracking issue: https://github.com/rust-lang/rust/issues/27812
decl_storage! {
	trait Store for Module<T: Trait> as Bridge {
		pub DepositPool get(deposit_pool): RingBalanceOf<T>;

		pub EOSBridge get(eos_bridge): EOSBridgeT;
		pub EOSDepositValue get(eos_deposit_value) config(): RingBalanceOf<T>;
		pub EOSRelayer get(eos_relayer): EOSRelayerT;

		pub EthereumBridge get(ethereum_bridge): EthereumBridgeT;
		pub EthereumDepositValue get(ethereum_deposit_value) config(): RingBalanceOf<T>;
		pub EthereumRelayer get(etnereum_relayer): EthereumRelayerT;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call
	where
		origin: T::Origin
	{
		pub fn submit_header(origin, target_chain: TargetChain, header: Header) {
			let _relayer = ensure_signed(origin)?;
			let _ = Self::verify(&header)?;

			{
				match target_chain {
					EOS => {}
					Ethereum => {}
				}
			}
		}

		// `Darwinia lock` corresponds to `TargetChain redeem`
		pub fn lock(origin, _target_chain: TargetChain) {
			let _locker = ensure_signed(origin)?;
		}

		// `Darwinia redeem` corresponds to `TargetChain lock`
		pub fn redeem(origin, _target_chain: TargetChain, _header: Header) {
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
	fn _adjust_deposit_value(_: TargetChain, _value: RingBalanceOf<T>) -> Result {
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

	fn _match_chain_with<F: FnOnce(&mut dyn Bridge)>(target_chain: TargetChain, f: F) {
		match target_chain {
			EOS => {
				EOSBridge::mutate(|eos_bridge| f(eos_bridge));
			}
			Ethereum => {}
		}

		unimplemented!()
	}
}

type RingBalanceOf<T> = <<T as Trait>::Ring as Currency<<T as system::Trait>::AccountId>>::Balance;
// TODO: type
type Header = ();

// FIXME: currently, use SPV instead
// pub type MMR = MerkleMountainRange<Blake2b>;

#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode)]
pub enum TargetChain {
	EOS,
	Ethereum,
}
