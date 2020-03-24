//! # Chain MMR Pallet
//!
//! ## Overview
//! This is the pallet to maintain accumulate headers Merkle Mountain Range
//! and push the mmr root in to the digest of block headers on finalize.
//! MMR can be used for light client to implement super light clients,
//! and can also be used in other chains to implement chain relay for
//! cross-chain verification purpose.
//!
//! ## Terminology
//!
//! ### Merkle Moutain Range
//! For more details about the MMR struct, refer https://github.com/mimblewimble/grin/blob/master/doc/mmr.md#structure
//!
//! ### MMR Proof
//! Using the MMR Store Storage, MMR Proof can be generated for specific
//! block header hash. Proofs can be used to verify block inclusion together with
//! the mmr root in the header digest.
//!
//! ### Positions
//! The index position of the nodes(and hash leave nodes) in the mmr node list
//! constructed using MMR struct
//!
//! ### Digest Item
//! The is a ```MerkleMountainRangeRoot(Hash)``` digest item pre-subscribed in Digest.
//! This is implemented in Darwinia's fork of substrate: https://github.com/darwinia-network/substrate
//! The Pull request link is https://github.com/darwinia-network/substrate/pull/1
//!
//! ## Implementation
//! We are using the MMR library from https://github.com/nervosnetwork/merkle-mountain-range
//! Pull request: https://github.com/darwinia-network/darwinia/pull/358
//!
//! ## References
//! Darwinia Relay's Technical Paper:
//! https://github.com/darwinia-network/rfcs/blob/master/paper/Darwinia_Relay_Sublinear_Optimistic_Relay_for_Interoperable_Blockchains_v0.7.pdf
//!
//! https://github.com/mimblewimble/grin/blob/master/doc/mmr.md#structure
//! https://github.com/mimblewimble/grin/blob/0ff6763ee64e5a14e70ddd4642b99789a1648a32/core/src/core/pmmr.rs#L606
//! https://github.com/nervosnetwork/merkle-mountain-range/blob/master/src/tests/test_accumulate_headers.rs
//! https://eprint.iacr.org/2019/226.pdf
//!

#![cfg_attr(not(feature = "std"), no_std)]

mod mock;
mod tests;

// --- third-party ---
use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure};
use frame_system::{self as system};
use merkle_mountain_range::{MMRStore, MerkleProof, MMR};
use sp_runtime::{generic::DigestItem, traits::Hash, DispatchError};
use sp_std::{marker::PhantomData, prelude::*};

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as HeaderMMR {
		/// MMR struct of the previous blocks, from first(genesis) to parent hash.
		pub MMRNodeList get(fn mmr_node_list): map hasher(identity) u64 => T::Hash;

		/// The MMR size and length of the mmr node list
		pub MMRCounter get(fn mmr_counter): u64;

		/// The positions of header numbers in the MMR Node List
		pub Positions get(fn position_of): map hasher(identity) T::BlockNumber => u64;
	}
}

decl_event! {
	pub enum Event<T>
	where
		H = <T as system::Trait>::Hash,
	{
		/// New mmr root log hash been deposited.
		NewMMRRoot(H),
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Proof block nubmer TOO LARGE
		ProofBlockNumberTL,
		/// Proof - GET FAILED
		ProofGF,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call
	where
		origin: T::Origin
	{
		type Error = Error<T>;

		fn deposit_event() = default;

		fn on_finalize(block_number: T::BlockNumber) {
			let store = <ModuleMMRStore<T>>::default();
			let mut mmr = <MMR<_, MMRMerge<T>, _>>::new(<MMRCounter>::get(), store);

			let parent_hash = <frame_system::Module<T>>::parent_hash();
			// Update MMR and add mmr root to digest of block header
			let pos = mmr.push(parent_hash).expect("Failed to push parent hash to mmr.");

			// The first block number should start with 1 and parent block should be (T::BlockNumber::zero(), hash69())
			// Checking just in case custom changes in system gensis config
			if block_number >= 1.into() {
				<Positions<T>>::insert(block_number - 1.into(), pos);
			}

			let mmr_root = mmr.get_root().expect("Failed to calculate merkle mountain range; qed");
			mmr.commit().expect("Failed to push parent hash to mmr.");

			let mmr_item = DigestItem::MerkleMountainRangeRoot(mmr_root.into());

			<frame_system::Module<T>>::deposit_log(mmr_item.into());
			Self::deposit_event(<Event<T>>::NewMMRRoot(mmr_root));
		}
	}
}

pub struct MMRMerge<T>(PhantomData<T>);
impl<T: Trait> merkle_mountain_range::Merge for MMRMerge<T> {
	type Item = <T as frame_system::Trait>::Hash;
	fn merge(lhs: &Self::Item, rhs: &Self::Item) -> Self::Item {
		let encodable = (lhs, rhs);
		<T as frame_system::Trait>::Hashing::hash_of(&encodable)
	}
}

pub struct ModuleMMRStore<T>(PhantomData<T>);
impl<T> Default for ModuleMMRStore<T> {
	fn default() -> Self {
		ModuleMMRStore(sp_std::marker::PhantomData)
	}
}

impl<T: Trait> MMRStore<T::Hash> for ModuleMMRStore<T> {
	fn get_elem(&self, pos: u64) -> merkle_mountain_range::Result<Option<T::Hash>> {
		Ok(Some(<Module<T>>::mmr_node_list(pos)))
	}

	fn append(&mut self, pos: u64, elems: Vec<T::Hash>) -> merkle_mountain_range::Result<()> {
		let mmr_count = MMRCounter::get();
		if pos != mmr_count {
			// Must be append only.
			Err(merkle_mountain_range::Error::InconsistentStore)?;
		}
		let elems_len = elems.len() as u64;

		for (i, elem) in elems.into_iter().enumerate() {
			<MMRNodeList<T>>::insert(mmr_count + i as u64, elem);
		}

		// increment counter
		MMRCounter::put(mmr_count + elems_len);

		Ok(())
	}
}

impl<T: Trait> Module<T> {
	// TODO: Add rpc call for this
	fn _gen_proof(
		block_number: T::BlockNumber,
		mmr_block_number: T::BlockNumber,
	) -> Result<MerkleProof<T::Hash, MMRMerge<T>>, DispatchError> {
		ensure!(block_number < mmr_block_number, <Error<T>>::ProofBlockNumberTL);

		let pos = Self::position_of(block_number);
		let mmr_header_pos = Self::position_of(mmr_block_number);

		let store = <ModuleMMRStore<T>>::default();
		let mmr = <MMR<_, MMRMerge<T>, _>>::new(mmr_header_pos, store);

		let proof = mmr.gen_proof(vec![pos]).map_err(|_| <Error<T>>::ProofGF)?;

		Ok(proof)
	}
}
