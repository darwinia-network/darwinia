//! # MMR Digest Pallet

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage};
use sp_std::marker::PhantomData;
use sp_std::prelude::*;
//use frame_benchmarking::{benchmarks, account};
//use frame_system::{self as system, ensure_signed, ensure_root, RawOrigin};

//use codec::{Encode, Decode};
use sp_runtime::{generic::DigestItem, traits::Hash};

use merkle_mountain_range::{MMRStore, MMR};

pub trait Trait: frame_system::Trait {}

decl_storage! {
	trait Store for Module<T: Trait> as ChainMMR {
		/// MMR struct of the previous blocks, from first(genesis) to parent hash.
		pub MMRList get(fn mmr_list): map hasher(twox_64_concat) u64 => T::Hash;
		pub MMRCounter get(fn mmr_counter): u64;
	}
}

//decl_event!(
//);
// `ensure_root` and `ensure_none`.

pub struct MMRMerge<T>(PhantomData<T>);

impl<T: Trait> merkle_mountain_range::Merge for MMRMerge<T> {
	type Item = <T as frame_system::Trait>::Hash;
	fn merge(lhs: &Self::Item, rhs: &Self::Item) -> Self::Item {
		let encoded_vec = vec![lhs, rhs];
		<T as frame_system::Trait>::Hashing::hash_of(&encoded_vec)
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
		Ok(Some(Module::<T>::mmr_list(pos)))
	}

	fn append(&mut self, pos: u64, elems: Vec<T::Hash>) -> merkle_mountain_range::Result<()> {
		let mmr_count = <MMRCounter>::get();
		if pos != mmr_count {
			// Must be append only.
			return Err(merkle_mountain_range::Error::InconsistentStore);
		}

		let elems_len = elems.len() as u64;

		for (i, elem) in elems.into_iter().enumerate() {
			<MMRList<T>>::insert(mmr_count + i as u64, elem);
		}

		// increment counter
		<MMRCounter>::put(mmr_count + elems_len);

		Ok(())
	}
}

decl_module! {
	// Simple declaration of the `Module` type. Lets the macro know what its working on.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// Deposit one of this pallet's events by using the default implementation.
		/// It is also possible to provide a custom implementation.
		/// For non-generic events, the generic parameter just needs to be dropped, so that it
		/// looks like: `fn deposit_event() = default;`.
//		fn deposit_event() = default;

		fn on_finalize(_block_number: T::BlockNumber) {
			let store = ModuleMMRStore::<T>::default();

			let mut mmr = MMR::<_, MMRMerge<T>, _>::new(<MMRCounter>::get(), store);

			let parent_hash = <frame_system::Module<T>>::parent_hash();
			// Update MMR and add mmr root to digest of block header
			mmr.push(parent_hash).expect("Failed to push parent hash to mmr.");

			let mmr_root = mmr.get_root().expect("Failed to calculate merkle mountain range; qed");

			mmr.commit().expect("Failed to push parent hash to mmr.");

			let mmr_item = DigestItem::MerkleMountainRangeRoot(
				mmr_root.into()
			);

			<frame_system::Module<T>>::deposit_log(mmr_item.into());
		}
	}
}

impl<T: Trait> Module<T> {
	// Nothing
}
