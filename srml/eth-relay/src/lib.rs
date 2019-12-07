//!  prototype module for bridging in ethereum poa blockcahin

#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

// use blake2::Blake2b;
use codec::{Decode, Encode};
use rstd::vec::Vec;
use sr_eth_primitives::{
	header::EthHeader, pow::EthashPartial, pow::EthashSeal, receipt::Receipt, BlockNumber as EthBlockNumber, H256, U256,
};

use ethash::{EthereumPatch, LightDAG};

use support::{decl_event, decl_module, decl_storage, dispatch::Result, ensure, traits::Get};

use system::ensure_signed;

use sr_primitives::RuntimeDebug;

use merkle_patricia_trie::{trie::Trie, MerklePatriciaTrie, Proof};

type DAG = LightDAG<EthereumPatch>;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	type EthNetwork: Get<u64>;
}

/// Familial details concerning a block
#[derive(Default, Clone, Copy, Eq, PartialEq, Encode, Decode)]
pub struct BlockDetails {
	/// Block number
	pub height: EthBlockNumber,
	pub hash: H256,
	/// Total difficulty of the block and all its parents
	pub total_difficulty: U256,
	//	/// Parent block hash
	//	pub parent: H256,
	//	/// List of children block hashes
	//	pub children: Vec<H256>,
	//	/// Whether the block is considered finalized
	//	pub is_finalized: bool,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct ActionRecord {
	pub index: u64,
	pub proof: Vec<u8>,
	pub header_hash: H256,
}

decl_storage! {
	trait Store for Module<T: Trait> as EthRelay {
		/// Anchor block that works as genesis block
		pub BeginHeader get(fn begin_header): Option<EthHeader>;

		/// Info of the best block header for now
		pub BestHeaderHash get(fn best_header_hash): H256;

		pub HeaderOf get(header_of): map H256 => Option<EthHeader>;

		pub HeaderDetailsOf get(header_details_of): map H256 => Option<BlockDetails>;

		/// Block delay for verify transaction
		pub FinalizeNumber get(finalize_number): Option<u64>;

		pub ActionOf get(action_of): map T::Hash => Option<ActionRecord>;

//		pub BestHashOf get(best_hash_of): map u64 => Option<H256>;

//		pub HashsOf get(hashs_of): map u64 => Vec<H256>;

//		pub HeaderForIndex get(header_for_index): map H256 => Vec<(u64, T::Hash)>;
	}
	add_extra_genesis {
		config(header): Option<Vec<u8>>;
		config(genesis_difficulty): u64;
		build(|config| {
			if let Some(h) = &config.header {
				let header: EthHeader = rlp::decode(&h).expect("Deserialize Header - FAILED");

				<Module<T>>::init_genesis_header(&header,config.genesis_difficulty);

				// TODO: initilize other parameters.
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

		pub fn reset_genesis_header(origin, header: EthHeader, genesis_difficulty: u64) {
			let _relayer = ensure_signed(origin)?;
			// TODO: Check authority

			// TODO: Just for easy testing.
			Self::init_genesis_header(&header, genesis_difficulty);

			<Module<T>>::deposit_event(RawEvent::NewHeader(header));
		}

		pub fn relay_header(origin, header: EthHeader) {
			let _relayer = ensure_signed(origin)?;
			// 1. There must be a corresponding parent hash
			// 2. Update best hash if the current block number is larger than current best block's number （Chain reorg）

			Self::verify_header(&header)?;

			Self::store_header(&header)?;

			<Module<T>>::deposit_event(RawEvent::NewHeader(header));
		}

		pub fn check_receipt(origin, proof_record: ActionRecord) {
			let _relayer = ensure_signed(origin)?;

			let verified_receipt = Self::verify_receipt(&proof_record);

			ensure!(verified_receipt.is_some(), "Receipt Proof Verification - FAILED");

			<Module<T>>::deposit_event(RawEvent::RelayProof(verified_receipt.unwrap(), proof_record));
		}

		// Assuming that there are at least one honest worker submiting headers
		// This method may be merged together with relay_header
		pub fn challenge_header(origin, _header: EthHeader) {
			let _relayer = ensure_signed(origin)?;
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
		RelayProof(Receipt, ActionRecord),
		TODO(AccountId),

//		 Develop
		//		Print(u64),
	}
}

impl<T: Trait> Module<T> {
	// TOOD: what is the total difficulty for genesis/begin header
	pub fn init_genesis_header(header: &EthHeader, genesis_difficulty: u64) {
		let header_hash = header.hash();
		let block_number = header.number();

		HeaderOf::insert(&header_hash, header);

		// initialize the header details, including total difficulty.
		HeaderDetailsOf::insert(
			&header_hash,
			BlockDetails {
				height: block_number,
				hash: header_hash,
				total_difficulty: genesis_difficulty.into(),
			},
		);

		// Initialize the the best hash.
		BestHeaderHash::mutate(|hash| {
			*hash = header_hash;
		});

		// Initialize the header.
		BeginHeader::put(header.clone());
	}

	fn verify_receipt(proof_record: &ActionRecord) -> Option<Receipt> {
		let header_hash = proof_record.header_hash;
		if !HeaderOf::exists(header_hash) {
			return None; //Err("This block header does not exist.");
		}

		let header = HeaderOf::get(header_hash).unwrap();

		let proof: Proof = rlp::decode(&proof_record.proof).unwrap();
		let key = rlp::encode(&proof_record.index);

		let value = MerklePatriciaTrie::verify_proof(header.receipts_root().0.to_vec(), &key, proof).unwrap();
		if !value.is_some() {
			return None;
		}

		let proof_receipt: Receipt = rlp::decode(&value.unwrap()).expect("Deserialize Receipt - FAILED");

		Some(proof_receipt)
		// confirm that the block hash is right
		// get the receipt MPT trie root from the block header
		// Using receipt MPT trie root to verify the proof and index etc.
	}

	/// 1. proof of difficulty
	/// 2. proof of pow (mixhash)
	/// 3. challenge
	fn verify_header(header: &EthHeader) -> Result {
		// TODO: check parent hash,
		let parent_hash = header.parent_hash();

		let number = header.number();

		ensure!(
			number >= Self::begin_header().expect("Begin Header - NOT EXISTED").number(),
			"Block Number - TOO SMALL"
		);

		let prev_header = Self::header_of(parent_hash).expect("Previous Header - NOT EXISTED");
		ensure!((prev_header.number() + 1) == number, "Block Number - NOT MATCHED");

		// check difficulty
		let ethash_params = match T::EthNetwork::get() {
			0 => EthashPartial::production(),
			1 => EthashPartial::ropsten_testnet(),
			_ => EthashPartial::production(), // others
		};
		ethash_params.verify_block_basic(header)?;

		// verify difficulty
		let difficulty = ethash_params.calculate_difficulty(header, &prev_header);
		ensure!(difficulty == *header.difficulty(), "Difficulty Verification - FAILED");

		// verify mixhash
		let seal = EthashSeal::parse_seal(header.seal())?;

		let light_dag = DAG::new(number.into());
		let partial_header_hash = header.bare_hash();
		let mix_hash = light_dag.hashimoto(partial_header_hash, seal.nonce).0;

		if mix_hash != seal.mix_hash {
			return Err("Mixhash - NOT MATCHED");
		}

		//			ensure!(best_header.height == block_number, "Block height does not match.");
		//			ensure!(best_header.hash == *header.parent_hash(), "Block hash does not match.");

		Ok(())
	}

	fn store_header(header: &EthHeader) -> Result {
		let header_hash = header.hash();
		let block_number = header.number();

		HeaderOf::insert(header_hash, header);

		let prev_total_difficulty = Self::header_details_of(header.parent_hash()).unwrap().total_difficulty;

		HeaderDetailsOf::insert(
			header_hash,
			BlockDetails {
				height: block_number,
				hash: header_hash,
				total_difficulty: prev_total_difficulty + header.difficulty(),
			},
		);

		let best_header_hash = Self::best_header_hash();
		//			let best_header = Self::header_of(best_header_hash).ok_or("Can not find best header.");
		let best_header_details = Self::header_details_of(best_header_hash).unwrap();

		// TODO: Check total difficulty and reorg if necessary.
		if prev_total_difficulty + header.difficulty() > best_header_details.total_difficulty {
			BestHeaderHash::mutate(|hash| {
				*hash = header_hash;
			});
		}

		Ok(())
	}

	fn _punish(_who: &T::AccountId) -> Result {
		unimplemented!()
	}
}
