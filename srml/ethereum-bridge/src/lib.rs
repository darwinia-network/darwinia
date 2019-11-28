//!  prototype module for bridging in ethereum poa blockcahin

#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

// use blake2::Blake2b;
use codec::{Decode, Encode};
use rstd::vec::Vec;
use sr_eth_primitives::{pow::EthHeader, BestBlock, H160, H256, H64, U128, U256, U512};
use support::{decl_event, decl_module, decl_storage, dispatch::Result};
use system::ensure_signed;

use sr_primitives::RuntimeDebug;

use rlp::{decode, encode};

//use web3::types::{
//	Address, Block, BlockId, BlockNumber, Bytes, CallRequest, Filter, Index, Log, RawHeader, RawReceipt, SyncState,
//	Transaction, TransactionId, TransactionReceipt, TransactionRequest, Work, H256, H520, H64, U128, U256,
//};

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	//	type Hash: rstd::hash::Hash;
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct Proof {
	pub nodes: Vec<Vec<u8>>,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct ActionRecord {
	pub index: u64,
	pub proof: Vec<u8>,
	pub header_hash: H256,
}

decl_storage! {
	trait Store for Module<T: Trait> as EthBridge {
		pub BeginNumber get(begin_number): u64;

		pub BeginHeader get(begin_header): Option<EthHeader>;

		pub BestHeader get(best_header): BestBlock;

		pub HeaderOf get(header_of): map H256 => Option<EthHeader>;

		pub BestHashOf get(best_hash_of): map u64 => Option<H256>;

		pub HashsOf get(hashs_of): map u64 => Vec<H256>;

		/// Block delay for verify transaction
		pub FinalizeNumber get(finalize_number): Option<u64>;

		pub ActionOf get(action_of): map T::Hash => Option<ActionRecord>;

		pub HeaderForIndex get(header_for_index): map H256 => Vec<(u64, T::Hash)>;
	}
	add_extra_genesis {
		config(header): Option<Vec<u8>>;
		config(number): u64;
		build(|config| {
			if let Some(h) = &config.header {
				let header: EthHeader = rlp::decode(&h).expect("can't deserialize the header");
				BeginNumber::put(header.number());

//				<Module<T>>::::genesis_header(header);
			} else {
				BeginNumber::put(config.number);
			}
		});
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call
	where
		origin: T::Origin
	{
		pub fn genesis_header(origin, header: EthHeader) {
			let _relayer = ensure_signed(origin)?;
//			BeginHeader::put(header);
		}

		pub fn store_block_header(origin, header: EthHeader) {
			let _relayer = ensure_signed(origin)?;
			let _ = Self::verify(&header)?;
		}

		pub fn relay_receipt(origin, proof: ActionRecord) {
			// confirm that the block hash is right
			// get the MPT from the block header
			// Using MPT to verify the proof and index etc.
		}

		pub fn submit_header(origin, header: EthHeader) {
			// if header confirmed then return
			// if header in unverified header then challenge
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
	/// 2. verify (difficulty + prev_hash + nonce + re-org)
	/// 3. challenge
	fn verify(_: &EthHeader) -> Result {
		unimplemented!()
	}

	fn _punish(_who: &T::AccountId) -> Result {
		unimplemented!()
	}
}
