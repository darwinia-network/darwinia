//! prototype module for bridging in ethereum pow blockchain, including mainet and ropsten.

#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", test))]
mod mock;
#[cfg(all(feature = "std", test))]
mod tests;

use codec::{Decode, Encode};
use frame_support::{decl_event, decl_module, decl_storage, ensure, traits::Get, weights::SimpleDispatchInfo};
use frame_system::{self as system, ensure_root, ensure_signed};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

use eth_primitives::{
	header::EthHeader, pow::EthashPartial, pow::EthashSeal, receipt::Receipt, EthBlockNumber, H256, U256,
};
use ethash::{EthereumPatch, LightDAG};
use merkle_patricia_trie::{trie::Trie, MerklePatriciaTrie, Proof};

type DAG = LightDAG<EthereumPatch>;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	type EthNetwork: Get<u64>;
}

/// Familial details concerning a block
#[derive(Default, Clone, Copy, Eq, PartialEq, Encode, Decode)]
pub struct HeaderInfo {
	/// Total difficulty of the block and all its parents
	pub total_difficulty: U256,
	/// Parent hash of the header
	pub parent_hash: H256,
	/// Block number
	pub number: EthBlockNumber,
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
		pub GenesisHeader get(fn begin_header): Option<EthHeader>;

		/// Hash of best block header
		pub BestHeaderHash get(fn best_header_hash): H256;

		pub CanonicalHeaderHashOf get(fn canonical_header_hash_of): map hasher(blake2_256) u64 => H256;

		pub HeaderOf get(fn header_of): map hasher(blake2_256) H256 => Option<EthHeader>;

		pub HeaderInfoOf get(fn header_info_of): map hasher(blake2_256) H256 => Option<HeaderInfo>;

		/// Number of blocks finality
		pub NumberOfBlocksFinality get(fn number_of_blocks_finality) config(): u64 = 30;

		pub NumberOfBlocksSafe get(fn number_of_blocks_safe) config(): u64 = 10;

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

		#[weight = SimpleDispatchInfo::FixedNormal(5_000)]
		fn set_number_of_blocks_finality(origin, #[compact] new: u64) {
			ensure_root(origin)?;
			NumberOfBlocksFinality::put(new);
		}

		#[weight = SimpleDispatchInfo::FixedNormal(5_000)]
		fn set_number_of_blocks_safe(origin, #[compact] new: u64) {
			ensure_root(origin)?;
			NumberOfBlocksSafe::put(new);
		}

		// TODO: Just for easy testing.
		pub fn reset_genesis_header(origin, header: EthHeader, genesis_difficulty: u64) {
			let relayer = ensure_signed(origin)?;
			if Self::check_authorities() {
				ensure!(Self::authorities().contains(&relayer), "Account - NO PRIVILEGES");
			}

			Self::init_genesis_header(&header, genesis_difficulty)?;

			<Module<T>>::deposit_event(RawEvent::SetGenesisHeader(relayer, header, genesis_difficulty));
		}

		pub fn relay_header(origin, header: EthHeader) {
			let relayer = ensure_signed(origin)?;
			if Self::check_authorities() {
				ensure!(Self::authorities().contains(&relayer), "Account - NO PRIVILEGES");
			}

			let header_hash = header.hash();

			ensure!(!HeaderInfoOf::get(&header_hash).is_some(), "The header is already known.");

//			let best_header_hash = Self::best_header_hash();
//			if self.best_header_hash == Default::default() {
//				Self::maybe_store_header(&header)?;
//			}

			Self::verify_header(&header)?;
			Self::maybe_store_header(&header)?;

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

		pub fn add_authority(origin, who: T::AccountId) {
			let _me = ensure_root(origin)?;

			if !Self::authorities().contains(&who) {
				<Authorities<T>>::mutate(|l| l.push(who.clone()));

				<Module<T>>::deposit_event(RawEvent::AddAuthority(who));
			}
		}

		pub fn remove_authority(origin, who: T::AccountId) {
			let _me = ensure_root(origin)?;

			if let Some(i) = Self::authorities()
				.into_iter()
				.position(|who_| who_ == who) {
				<Authorities<T>>::mutate(|l| l.remove(i));

				<Module<T>>::deposit_event(RawEvent::RemoveAuthority(who));
			}
		}

		pub fn toggle_check_authorities(origin) {
			let _me = ensure_root(origin)?;

			CheckAuthorities::put(!Self::check_authorities());

			<Module<T>>::deposit_event(RawEvent::ToggleCheckAuthorities(Self::check_authorities()));
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
		AddAuthority(AccountId),
		RemoveAuthority(AccountId),
		ToggleCheckAuthorities(bool),
	}
}

/// Handler for selecting the genesis validator set.
pub trait VerifyEthReceipts {
	fn verify_receipt(proof_record: &EthReceiptProof) -> Result<Receipt, &'static str>;
}

impl<T: Trait> Module<T> {
	pub fn init_genesis_header(header: &EthHeader, genesis_difficulty: u64) -> Result<(), &'static str> {
		let header_hash = header.hash();

		ensure!(header_hash == header.re_compute_hash(), "Header Hash - MISMATCHED");

		let block_number = header.number();

		HeaderOf::insert(&header_hash, header);

		// initialize header info, including total difficulty.
		HeaderInfoOf::insert(
			&header_hash,
			HeaderInfo {
				parent_hash: *header.parent_hash(),
				total_difficulty: genesis_difficulty.into(),
				number: block_number,
			},
		);

		// Initialize the the best hash.
		BestHeaderHash::mutate(|hash| {
			*hash = header_hash;
		});

		CanonicalHeaderHashOf::insert(block_number, &header_hash);

		// Removing headers with larger numbers, if there are.
		let mut number = block_number + 1;
		loop {
			// If the current block hash is 0 (unlikely), or the previous hash matches the
			// current hash, then we chains converged and can stop now.
			if !CanonicalHeaderHashOf::contains_key(&number) {
				break;
			}

			CanonicalHeaderHashOf::remove(&number);
			number += 1;
		}

		GenesisHeader::put(header.clone());

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

		// There must be a corresponding parent hash
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

	fn maybe_store_header(header: &EthHeader) -> Result<(), &'static str> {
		let best_header_info =
			Self::header_info_of(Self::best_header_hash()).ok_or("Best Header Detail - NOT EXISTED")?;

		if best_header_info.number > header.number + Self::number_of_blocks_finality() {
			return Err("Header Too Old: It's too late to add this block header.");
		}

		let header_hash = header.hash();
		HeaderOf::insert(header_hash, header);

		let parent_total_difficulty = Self::header_info_of(header.parent_hash())
			.ok_or("Previous Header Detail - NOT EXISTED")?
			.total_difficulty;

		let block_number = header.number();
		let header_info = HeaderInfo {
			number: block_number,
			parent_hash: *header.parent_hash(),
			total_difficulty: parent_total_difficulty + header.difficulty(),
		};

		HeaderInfoOf::insert(&header_hash, header_info);

		// Check total difficulty and re-org if necessary.
		if header_info.total_difficulty > best_header_info.total_difficulty
			|| (header_info.total_difficulty == best_header_info.total_difficulty
				&& header.difficulty % 2 == U256::default())
		{
			// The new header is the tip of the new canonical chain.
			// We need to update hashes of the canonical chain to match the new header.

			// If the new header has a lower number than the previous header, we need to cleaning
			// it going forward.
			if best_header_info.number > header_info.number {
				for number in header_info.number + 1..=best_header_info.number {
					CanonicalHeaderHashOf::remove(&number);
				}
			}
			// Replacing the global best header hash.
			BestHeaderHash::mutate(|hash| {
				*hash = header_hash;
			});

			CanonicalHeaderHashOf::insert(&header_info.number, &header_hash);

			// Replacing past hashes until we converge into the same parent.
			// Starting from the parent hash.
			let mut number = header.number - 1;
			let mut current_hash = header_info.parent_hash;
			loop {
				let prev_value = CanonicalHeaderHashOf::get(&number);
				// If the current block hash is 0 (unlikely), or the previous hash matches the
				// current hash, then we chains converged and can stop now.
				if number == 0 || prev_value == current_hash {
					break;
				}

				CanonicalHeaderHashOf::insert(&number, &current_hash);

				// Check if there is an info to get the parent hash
				if let Some(info) = HeaderInfoOf::get(&current_hash) {
					current_hash = info.parent_hash;
				} else {
					break;
				}
				number -= 1;
			}
		}

		Ok(())
	}

	// TODO: Economic model design required for the relay
	fn _punish(_who: &T::AccountId) -> Result<(), &'static str> {
		unimplemented!()
	}
}

impl<T: Trait> VerifyEthReceipts for Module<T> {
	/// confirm that the block hash is right
	/// get the receipt MPT trie root from the block header
	/// Using receipt MPT trie root to verify the proof and index etc.
	fn verify_receipt(proof_record: &EthReceiptProof) -> Result<Receipt, &'static str> {
		let info = Self::header_info_of(&proof_record.header_hash).ok_or("Header - NOT EXISTED")?;
		let canonical_hash = Self::canonical_header_hash_of(info.number);
		if proof_record.header_hash != canonical_hash {
			return Err("Header - NOT CANONICAL");
		}

		let best_info = Self::header_info_of(Self::best_header_hash()).ok_or("Header - Best Header Not Found")?;
		if best_info.number < info.number + Self::number_of_blocks_safe() {
			return Err("Header - NOT SAFE");
		}

		let header = Self::header_of(&proof_record.header_hash).ok_or("Header - NOT EXISTED")?;
		let proof: Proof = rlp::decode(&proof_record.proof).map_err(|_| "Rlp Decode - FAILED")?;
		let key = rlp::encode(&proof_record.index);
		let value = MerklePatriciaTrie::verify_proof(header.receipts_root().0.to_vec(), &key, proof)
			.map_err(|_| "Verify Proof - FAILED")?
			.ok_or("Trie Key - NOT EXISTED")?;
		let receipt = rlp::decode(&value).map_err(|_| "Deserialize Receipt - FAILED")?;

		Ok(receipt)
	}
}
