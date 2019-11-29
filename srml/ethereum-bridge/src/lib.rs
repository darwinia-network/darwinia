//!  prototype module for bridging in ethereum poa blockcahin

#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

// use blake2::Blake2b;
use codec::{Decode, Encode};
use rstd::vec::Vec;
use sr_eth_primitives::{
	header::EthHeader, pow::EthashPartial, pow::EthashSeal, receipt::Receipt, BestBlock, BlockNumber as EthBlockNumber,
	H160, H256, H64, U128, U256, U512,
};

use ethash::{EthereumPatch, LightDAG};

use support::{decl_event, decl_module, decl_storage, dispatch::Result, ensure, traits::Currency};

use system::ensure_signed;

use sr_primitives::RuntimeDebug;

use rlp::{decode, encode};

use merkle_patricia_trie::{trie::Trie, MerklePatriciaTrie, Proof};

type DAG = LightDAG<EthereumPatch>;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	//	type Hash: rstd::hash::Hash;
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct ActionRecord {
	pub index: u64,
	pub proof: Vec<u8>,
	pub header_hash: H256,
}

decl_storage! {
	trait Store for Module<T: Trait> as EthBridge {
		/// Anchor block that works as genesis block
		pub BeginHeader get(fn begin_header): Option<EthHeader>;
		/// Info of the best block header for now
		pub BestHeader get(fn best_header): BestBlock;
		///
		pub BlockList get(fn block_list): map EthBlockNumber => Option<EthHeader>;
		/// nonce of each block
		pub Nonce get(fn nonce): map EthBlockNumber => Option<H64>;

		pub HeaderOf get(header_of): map H256 => Option<EthHeader>;

//		pub BestHashOf get(best_hash_of): map u64 => Option<H256>;
//		pub HashsOf get(hashs_of): map u64 => Vec<H256>;

		/// Block delay for verify transaction
		pub FinalizeNumber get(finalize_number): Option<u64>;

		pub ActionOf get(action_of): map T::Hash => Option<ActionRecord>;

		pub HeaderForIndex get(header_for_index): map H256 => Vec<(u64, T::Hash)>;
	}
	add_extra_genesis {
		config(header): Option<Vec<u8>>;
		build(|config| {
			if let Some(h) = &config.header {
				let header: EthHeader = rlp::decode(&h).expect("can't deserialize the header");
				BeginHeader::put(header.clone());

				<Module<T>>::genesis_header(header);
			}
		});
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call
	where
		origin: T::Origin
	{
		fn deposit_event() = default;

		pub fn relay_header(origin, header: EthHeader) {
			let _relayer = ensure_signed(origin)?;
			Self::verify_header(&header)?;

			<Module<T>>::deposit_event(RawEvent::NewHeader(header));
		}

		pub fn check_receipt(origin, receipt: Receipt, proof_record: ActionRecord) {
			let _relayer = ensure_signed(origin)?;

			let header_hash = proof_record.header_hash;
			if !HeaderOf::exists(header_hash) {
				return Err("This block header does not exist.")
			}

			let header = HeaderOf::get(header_hash).unwrap();

			let proof: Proof = rlp::decode(&proof_record.proof).unwrap();
			let key = rlp::encode(&proof_record.index);

			let value = MerklePatriciaTrie::verify_proof(header.receipts_root().0.to_vec(), &key, proof)
				.unwrap();
			assert!(value.is_some());

			let receipt_encoded = rlp::encode(&receipt);

			assert_eq!(value.unwrap(), receipt_encoded);
			// confirm that the block hash is right
			// get the receipt MPT trie root from the block header
			// Using receipt MPT trie root to verify the proof and index etc.

			<Module<T>>::deposit_event(RawEvent::RelayProof(proof_record));
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
		NewHeader(EthHeader),
		RelayProof(ActionRecord),
		TODO(AccountId),
	}
}

impl<T: Trait> Module<T> {
	pub fn genesis_header(header: EthHeader) {
		unimplemented!()
	}

	pub fn adjust_deposit_value() {
		unimplemented!()
	}

	/// 1. proof of difficulty
	/// 2. proof of pow (mixhash)
	/// 3. challenge
	fn verify_header(header: &EthHeader) -> Result {
		// check parent hash,
		let parent_hash = header.parent_hash();

		let number = header.number();
		ensure!(
			number >= Self::begin_header().unwrap().number(),
			"block nubmer is too small."
		);
		let prev_header = Self::block_list(number - 1).ok_or("import ancestor block first.");

		match HeaderOf::get(parent_hash) {
			None => return Err("Parent Header Not Found."),
			Some(parent_header) => {
				if parent_header.hash() != *parent_hash {
					return Err("Parent hash does not match.");
				}
			}
		}

		// check difficulty
		let mut ethash_params = EthashPartial::expanse();
		ethash_params.set_difficulty_bomb_delays(0xc3500, 5000000);
		let result = ethash_params.verify_block_basic(header);
		match result {
			Ok(_) => (),
			Err(e) => {
				return Err("Block difficulty verification failed.");
			}
		};

		// verify difficulty
		//		let difficulty = ethash_params.calculate_difficulty(header, &prev_header);
		//		ensure!(difficulty == header.difficulty(), "difficulty verification failed");

		// verify mixhash
		let seal = match EthashSeal::parse_seal(header.seal()) {
			Err(e) => {
				return Err("Seal parse error.");
			}
			Ok(x) => x,
		};

		let light_dag = DAG::new(number.into());
		let partial_header_hash = header.bare_hash();
		let mix_hash = light_dag.hashimoto(partial_header_hash, seal.nonce).0;

		if mix_hash != seal.mix_hash {
			return Err("Mixhash does not match.");
		}

		let header_hash = header.hash();

		HeaderOf::insert(header_hash, header);

		Ok(())
	}

	fn _punish(_who: &T::AccountId) -> Result {
		unimplemented!()
	}
}
