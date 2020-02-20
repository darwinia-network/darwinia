//!  prototype module for bridging in ethereum pow blockchain, including mainet and ropsten

#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", test))]
mod mock;
#[cfg(all(feature = "std", test))]
mod tests;

use codec::{Decode, Encode};
use frame_support::{decl_event, decl_module, decl_storage, ensure, traits::Get};
use frame_system::{self as system, ensure_root, ensure_signed};
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

use ethash::{EthereumPatch, LightDAG};
use merkle_patricia_trie::{trie::Trie, MerklePatriciaTrie, Proof};
use sr_eth_primitives::{
	header::EthHeader, pow::EthashPartial, pow::EthashSeal, receipt::Receipt, EthBlockNumber, H256, U256,
};

type DAG = LightDAG<EthereumPatch>;

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
pub struct EthReceiptProof {
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

//		pub BestHashOf get(best_hash_of): map u64 => Option<H256>;

//		pub HashsOf get(hashs_of): map u64 => Vec<H256>;

//		pub HeaderForIndex get(header_for_index): map H256 => Vec<(u64, T::Hash)>;
//		pub UnverifiedHeader get(unverified_header): map PrevHash => Vec<Header>;

		pub CheckAuthorities get(fn check_authorities) config(): bool = true;
		pub Authorities get(fn authorities) config(): Vec<T::AccountId>;
	}
	add_extra_genesis {
		config(header): Option<Vec<u8>>;
		config(genesis_difficulty): u64;
		build(|config| {
			if let Some(h) = &config.header {
				let header: EthHeader = rlp::decode(&h).expect("Deserialize Genesis Header - FAILED");

				// Discard the result even it fail.
				let _ = <Module<T>>::init_genesis_header(&header,config.genesis_difficulty);

				// TODO: initialize other parameters.
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
			let relayer = ensure_signed(origin)?;
			if Self::check_authorities() {
				ensure!(Self::authorities().contains(&relayer), "Account - NO PRIVILEGES");
			}

			// TODO: Just for easy testing.
			Self::init_genesis_header(&header, genesis_difficulty)?;

			<Module<T>>::deposit_event(RawEvent::SetGenesisHeader(relayer, header, genesis_difficulty));
		}

		pub fn relay_header(origin, header: EthHeader) {
			let relayer = ensure_signed(origin)?;
			if Self::check_authorities() {
				ensure!(Self::authorities().contains(&relayer), "Account - NO PRIVILEGES");
			}
			// 1. There must be a corresponding parent hash
			// 2. Update best hash if the current block number is larger than current best block's number （Chain reorg）

			Self::verify_header(&header)?;

			Self::store_header(&header)?;

			<Module<T>>::deposit_event(RawEvent::RelayHeader(relayer, header));
		}

		pub fn check_receipt(origin, proof_record: EthReceiptProof) {
			let relayer = ensure_signed(origin)?;
			if Self::check_authorities() {
				ensure!(Self::authorities().contains(&relayer), "Account - NO PRIVILEGES");
			}

			let verified_receipt = Self::verify_receipt(&proof_record)?;

			<Module<T>>::deposit_event(RawEvent::VerifyProof(relayer, verified_receipt, proof_record));
		}

		// Assuming that there are at least one honest worker submiting headers
		// This method may be merged together with relay_header
		pub fn challenge_header(origin, _header: EthHeader) {
			let _relayer = ensure_signed(origin)?;
			// if header confirmed then return
			// if header in unverified header then challenge
		}

		pub fn add_authority(origin, who: T::AccountId) {
			let _me = ensure_root(origin)?;

			if !Self::authorities().contains(&who) {
				<Authorities<T>>::mutate(|l| l.push(who));
			}
		}

		pub fn remove_authority(origin, who: T::AccountId) {
			let _me = ensure_root(origin)?;

			if let Some(i) = Self::authorities()
				.into_iter()
				.position(|who_| who_ == who) {
				<Authorities<T>>::mutate(|l| l.remove(i));
			}
		}

		pub fn toggle_check_authorities(origin) {
			let _me = ensure_root(origin)?;

			CheckAuthorities::put(!Self::check_authorities());
		}
	}
}

decl_event! {
	pub enum Event<T>
	where
		<T as system::Trait>::AccountId
	{
		SetGenesisHeader(AccountId, EthHeader, u64),
		RelayHeader(AccountId, EthHeader),
		VerifyProof(AccountId, Receipt, EthReceiptProof),

		// Develop
		// Print(u64),
	}
}

/// Handler for selecting the genesis validator set.
pub trait VerifyEthReceipts {
	fn verify_receipt(proof_record: &EthReceiptProof) -> Result<Receipt, &'static str>;
}

impl<T: Trait> Module<T> {
	// TOOD: what is the total difficulty for genesis/begin header
	pub fn init_genesis_header(header: &EthHeader, genesis_difficulty: u64) -> Result<(), &'static str> {
		let header_hash = header.hash();

		ensure!(header_hash == header.re_compute_hash(), "Header Hash - MISMATCHED");

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

		Ok(())
	}

	/// 1. proof of difficulty
	/// 2. proof of pow (mixhash)
	/// 3. challenge
	fn verify_header(header: &EthHeader) -> Result<(), &'static str> {
		ensure!(header.hash() == header.re_compute_hash(), "Header Hash - MISMATCHED");

		let parent_hash = header.parent_hash();

		let number = header.number();

		ensure!(
			number >= Self::begin_header().ok_or("Begin Header - NOT EXISTED")?.number(),
			"Block Number - TOO SMALL",
		);

		// TODO: check parent hash is the last header, ignore or reorg
		let prev_header = Self::header_of(parent_hash).ok_or("Previous Header - NOT EXISTED")?;
		ensure!((prev_header.number() + 1) == number, "Block Number - MISMATCHED");

		// check difficulty
		let ethash_params = match T::EthNetwork::get() {
			0 => EthashPartial::production(),
			1 => EthashPartial::ropsten_testnet(),
			_ => EthashPartial::production(), // others
		};
		ethash_params.verify_block_basic(header)?;

		// verify difficulty
		let difficulty = ethash_params.calculate_difficulty(header, &prev_header);
		ensure!(difficulty == *header.difficulty(), "Verify Difficulty - FAILED");

		// verify mixhash
		match T::EthNetwork::get() {
			1 => {
				// TODO: Ropsten have issues, do not verify mixhash
			}
			_ => {
				let seal = EthashSeal::parse_seal(header.seal())?;

				let light_dag = DAG::new(number.into());
				let partial_header_hash = header.bare_hash();
				let mix_hash = light_dag.hashimoto(partial_header_hash, seal.nonce).0;

				if mix_hash != seal.mix_hash {
					return Err("Mixhash - MISMATCHED");
				}
			}
		};

		Ok(())
	}

	fn store_header(header: &EthHeader) -> Result<(), &'static str> {
		let header_hash = header.hash();
		let block_number = header.number();

		let prev_total_difficulty = Self::header_details_of(header.parent_hash())
			.ok_or("Previous Header Detail - NOT EXISTED")?
			.total_difficulty;
		let best_header_hash = Self::best_header_hash();
		//			let best_header = Self::header_of(best_header_hash).ok_or("Can not find best header.");
		let best_header_details =
			Self::header_details_of(best_header_hash).ok_or("Best Header Detail - NOT EXISTED")?;

		HeaderOf::insert(header_hash, header);

		HeaderDetailsOf::insert(
			header_hash,
			BlockDetails {
				height: block_number,
				hash: header_hash,
				total_difficulty: prev_total_difficulty + header.difficulty(),
			},
		);

		// TODO: Check total difficulty and reorg if necessary.
		if prev_total_difficulty + header.difficulty() > best_header_details.total_difficulty {
			BestHeaderHash::mutate(|hash| {
				*hash = header_hash;
			});
		}

		Ok(())
	}

	fn _punish(_who: &T::AccountId) -> Result<(), &'static str> {
		unimplemented!()
	}
}

impl<T: Trait> VerifyEthReceipts for Module<T> {
	fn verify_receipt(proof_record: &EthReceiptProof) -> Result<Receipt, &'static str> {
		let header = Self::header_of(&proof_record.header_hash).ok_or("Header - NOT EXISTED")?;
		let proof: Proof = rlp::decode(&proof_record.proof).map_err(|_| "Rlp Decode - FAILED")?;
		let key = rlp::encode(&proof_record.index);
		let value = MerklePatriciaTrie::verify_proof(header.receipts_root().0.to_vec(), &key, proof)
			.map_err(|_| "Verify Proof - FAILED")?
			.ok_or("Trie Key - NOT EXISTED")?;
		let receipt = rlp::decode(&value).map_err(|_| "Deserialize Receipt - FAILED")?;

		Ok(receipt)
		// confirm that the block hash is right
		// get the receipt MPT trie root from the block header
		// Using receipt MPT trie root to verify the proof and index etc.
	}
}
