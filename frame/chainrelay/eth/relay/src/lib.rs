//! prototype module for bridging in ethereum pow blockchain, including mainnet and ropsten.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "128"]

#[cfg(all(feature = "std", test))]
mod mock;
#[cfg(all(feature = "std", test))]
mod tests;

// --- third-party ---
use codec::{Decode, Encode};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, ensure, traits::Get, weights::SimpleDispatchInfo,
};
use frame_system::{self as system, ensure_root, ensure_signed};
use sp_runtime::{DispatchError, DispatchResult, RuntimeDebug};
use sp_std::prelude::*;

// --- custom ---
use eth_primitives::{
	header::EthHeader, pow::EthashPartial, pow::EthashSeal, receipt::Receipt, EthBlockNumber, H256, U256,
};
use ethash::{EthereumPatch, LightDAG};
use merkle_patricia_trie::{trie::Trie, MerklePatriciaTrie, Proof};

type DAG = LightDAG<EthereumPatch>;

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	type EthNetwork: Get<u64>;
}

/// Familial details concerning a block
#[derive(Clone, Default, PartialEq, Encode, Decode)]
pub struct HeaderInfo {
	/// Total difficulty of the block and all its parents
	pub total_difficulty: U256,
	/// Parent hash of the header
	pub parent_hash: H256,
	/// Block number
	pub number: EthBlockNumber,
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
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

		pub CanonicalHeaderHashOf get(fn canonical_header_hash_of): map hasher(blake2_128_concat) u64 => H256;

		pub HeaderOf get(fn header_of): map hasher(blake2_128_concat) H256 => Option<EthHeader>;

		pub HeaderInfoOf get(fn header_info_of): map hasher(blake2_128_concat) H256 => Option<HeaderInfo>;

		/// Number of blocks finality
		pub NumberOfBlocksFinality get(fn number_of_blocks_finality) config(): u64;
		pub NumberOfBlocksSafe get(fn number_of_blocks_safe) config(): u64;

		pub CheckAuthorities get(fn check_authorities) config(): bool = true;
		pub Authorities get(fn authorities) config(): Vec<T::AccountId>;
	}
	add_extra_genesis {
		config(header): Option<Vec<u8>>;
		config(genesis_difficulty): u64;
		build(|config| {
			if let Some(h) = &config.header {
				let header: EthHeader = rlp::decode(&h).expect(<Error<T>>::RlpDcF.into());
				// Discard the result even it fail.
				let _ = <Module<T>>::init_genesis_header(&header,config.genesis_difficulty);
			}
		});
	}
}

decl_event! {
	pub enum Event<T>
	where
		<T as frame_system::Trait>::AccountId
	{
		SetGenesisHeader(AccountId, EthHeader, u64),
		RelayHeader(AccountId, EthHeader),
		VerifyProof(AccountId, Receipt, EthReceiptProof),
		AddAuthority(AccountId),
		RemoveAuthority(AccountId),
		ToggleCheckAuthorities(bool),
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Account - NO PRIVILEGES
		AccountNP,

		/// Block Number - OVERFLOW
		BlockNumberOF,
		/// Block Number - UNDERFLOW
		BlockNumberUF,

		/// Block Number - MISMATCHED
		BlockNumberMis,
		/// Header Hash - MISMATCHED
		HeaderHashMis,
		/// Mixhash - MISMATCHED
		MixhashMis,

		/// Begin Header - NOT EXISTS
		BeginHeaderNE,
		/// Header - NOT EXISTS
		HeaderNE,
		/// Header Info - NOT EXISTS
		HeaderInfoNE,
		/// Trie Key - NOT EXISTS
		TrieKeyNE,

		/// Header - ALREADY EXISTS
		HeaderAE,
		/// Header - NOT CANONICAL
		HeaderNC,
		/// Header - NOT SAFE
		HeaderNS,
		/// Header - TOO EARLY
		HeaderTE,
		/// Header - TOO OLD,
		HeaderTO,

		/// Rlp - DECODE FAILED
		RlpDcF,
		/// Receipt - DESERIALIZE FAILED
		ReceiptDsF,
		/// Seal - PARSING FAILED
		SealPF,
		/// Block Basic - VERIFICATION FAILED
		BlockBasicVF,
		/// Difficulty - VERIFICATION FAILED
		DifficultyVF,
		/// Proof - VERIFICATION FAILED
		ProofVF,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call
	where
		origin: T::Origin
	{
		type Error = Error<T>;

		fn deposit_event() = default;

		// TODO: Just for easy testing.
		pub fn reset_genesis_header(origin, header: EthHeader, genesis_difficulty: u64) {
			let relayer = ensure_signed(origin)?;
			if Self::check_authorities() {
				ensure!(Self::authorities().contains(&relayer), <Error<T>>::AccountNP);
			}

			Self::init_genesis_header(&header, genesis_difficulty)?;

			<Module<T>>::deposit_event(RawEvent::SetGenesisHeader(relayer, header, genesis_difficulty));
		}

		pub fn relay_header(origin, header: EthHeader) {
			let relayer = ensure_signed(origin)?;
			if Self::check_authorities() {
				ensure!(Self::authorities().contains(&relayer), <Error<T>>::AccountNP);
			}

			let header_hash = header.hash();

			ensure!(HeaderInfoOf::get(&header_hash).is_none(), <Error<T>>::HeaderAE);

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
				ensure!(Self::authorities().contains(&relayer), <Error<T>>::AccountNP);
			}

			let verified_receipt = Self::verify_receipt(&proof_record)?;

			<Module<T>>::deposit_event(RawEvent::VerifyProof(relayer, verified_receipt, proof_record));
		}

		// --- root call ---

		pub fn add_authority(origin, who: T::AccountId) {
			ensure_root(origin)?;

			if !Self::authorities().contains(&who) {
				<Authorities<T>>::mutate(|l| l.push(who.clone()));

				<Module<T>>::deposit_event(RawEvent::AddAuthority(who));
			}
		}

		pub fn remove_authority(origin, who: T::AccountId) {
			ensure_root(origin)?;

			if let Some(i) = Self::authorities()
				.into_iter()
				.position(|who_| who_ == who) {
				<Authorities<T>>::mutate(|l| l.remove(i));

				<Module<T>>::deposit_event(RawEvent::RemoveAuthority(who));
			}
		}

		pub fn toggle_check_authorities(origin) {
			ensure_root(origin)?;

			CheckAuthorities::put(!Self::check_authorities());

			<Module<T>>::deposit_event(RawEvent::ToggleCheckAuthorities(Self::check_authorities()));
		}

		#[weight = SimpleDispatchInfo::FixedNormal(5_000)]
		pub fn set_number_of_blocks_finality(origin, #[compact] new: u64) {
			ensure_root(origin)?;
			NumberOfBlocksFinality::put(new);
		}

		#[weight = SimpleDispatchInfo::FixedNormal(5_000)]
		pub fn set_number_of_blocks_safe(origin, #[compact] new: u64) {
			ensure_root(origin)?;
			NumberOfBlocksSafe::put(new);
		}

	}
}

impl<T: Trait> Module<T> {
	pub fn init_genesis_header(header: &EthHeader, genesis_difficulty: u64) -> DispatchResult {
		let header_hash = header.hash();

		ensure!(header_hash == header.re_compute_hash(), <Error<T>>::HeaderHashMis);

		let block_number = header.number;

		HeaderOf::insert(&header_hash, header);

		// initialize header info, including total difficulty.
		HeaderInfoOf::insert(
			&header_hash,
			HeaderInfo {
				parent_hash: header.parent_hash,
				total_difficulty: genesis_difficulty.into(),
				number: block_number,
			},
		);

		// Initialize the the best hash.
		BestHeaderHash::put(header_hash);

		CanonicalHeaderHashOf::insert(block_number, header_hash);

		// Removing headers with larger numbers, if there are.
		for number in block_number.checked_add(1).ok_or(<Error<T>>::BlockNumberOF)?..u64::max_value() {
			// If the current block hash is 0 (unlikely), or the previous hash matches the
			// current hash, then we chains converged and can stop now.
			if !CanonicalHeaderHashOf::contains_key(&number) {
				break;
			}

			CanonicalHeaderHashOf::remove(&number);
		}

		GenesisHeader::put(header.clone());

		Ok(())
	}

	/// 1. proof of difficulty
	/// 2. proof of pow (mixhash)
	/// 3. challenge
	fn verify_header(header: &EthHeader) -> DispatchResult {
		ensure!(header.hash() == header.re_compute_hash(), <Error<T>>::HeaderHashMis);

		let begin_header_number = Self::begin_header().ok_or(<Error<T>>::BeginHeaderNE)?.number;
		ensure!(header.number >= begin_header_number, <Error<T>>::HeaderTE);

		// There must be a corresponding parent hash
		let prev_header = Self::header_of(header.parent_hash).ok_or(<Error<T>>::HeaderNE)?;
		ensure!(
			header.number == prev_header.number.checked_add(1).ok_or(<Error<T>>::BlockNumberOF)?,
			<Error<T>>::BlockNumberMis,
		);

		// check difficulty
		let ethash_params = match T::EthNetwork::get() {
			0 => EthashPartial::production(),
			1 => EthashPartial::ropsten_testnet(),
			_ => EthashPartial::production(), // others
		};
		ethash_params
			.verify_block_basic(header)
			.map_err(|_| <Error<T>>::BlockBasicVF)?;

		// verify difficulty
		let difficulty = ethash_params.calculate_difficulty(header, &prev_header);
		ensure!(difficulty == *header.difficulty(), <Error<T>>::DifficultyVF);

		// verify mixhash
		match T::EthNetwork::get() {
			1 => {
				// TODO: Ropsten have issues, do not verify mixhash
			}
			_ => {
				let seal = EthashSeal::parse_seal(header.seal()).map_err(|_| <Error<T>>::SealPF)?;

				let light_dag = DAG::new(header.number.into());
				let partial_header_hash = header.bare_hash();
				let mix_hash = light_dag.hashimoto(partial_header_hash, seal.nonce).0;

				ensure!(mix_hash == seal.mix_hash, <Error<T>>::MixhashMis);
			}
		};

		Ok(())
	}

	fn maybe_store_header(header: &EthHeader) -> DispatchResult {
		let best_header_info = Self::header_info_of(Self::best_header_hash()).ok_or(<Error<T>>::HeaderInfoNE)?;

		ensure!(
			best_header_info.number
				<= header
					.number
					.checked_add(Self::number_of_blocks_finality())
					.ok_or(<Error<T>>::BlockNumberOF)?,
			<Error<T>>::HeaderTO,
		);

		let parent_total_difficulty = Self::header_info_of(header.parent_hash)
			.ok_or(<Error<T>>::HeaderInfoNE)?
			.total_difficulty;

		let header_hash = header.hash();
		let header_info = HeaderInfo {
			number: header.number,
			parent_hash: header.parent_hash,
			total_difficulty: parent_total_difficulty
				.checked_add(header.difficulty)
				.ok_or(<Error<T>>::BlockNumberOF)?,
		};

		// Check total difficulty and re-org if necessary.
		if header_info.total_difficulty > best_header_info.total_difficulty
			|| (header_info.total_difficulty == best_header_info.total_difficulty
				&& header.difficulty % 2 == U256::zero())
		{
			// The new header is the tip of the new canonical chain.
			// We need to update hashes of the canonical chain to match the new header.

			// If the new header has a lower number than the previous header, we need to cleaning
			// it going forward.
			if best_header_info.number > header_info.number {
				for number in
					header_info.number.checked_add(1).ok_or(<Error<T>>::BlockNumberOF)?..=best_header_info.number
				{
					CanonicalHeaderHashOf::remove(&number);
				}
			}
			// Replacing the global best header hash.
			BestHeaderHash::put(header_hash);

			CanonicalHeaderHashOf::insert(header_info.number, header_hash);

			// Replacing past hashes until we converge into the same parent.
			// Starting from the parent hash.
			let mut current_hash = header_info.parent_hash;
			for number in (0..=header.number.checked_sub(1).ok_or(<Error<T>>::BlockNumberUF)?).rev() {
				let prev_value = CanonicalHeaderHashOf::get(number);
				// If the current block hash is 0 (unlikely), or the previous hash matches the
				// current hash, then we chains converged and can stop now.
				if number == 0 || prev_value == current_hash {
					break;
				}

				CanonicalHeaderHashOf::insert(number, current_hash);

				// Check if there is an info to get the parent hash
				if let Some(info) = HeaderInfoOf::get(current_hash) {
					current_hash = info.parent_hash;
				} else {
					break;
				}
			}
		}

		HeaderOf::insert(header_hash, header);
		HeaderInfoOf::insert(header_hash, header_info.clone());

		Ok(())
	}

	// TODO: Economic model design required for the relay
	fn _punish(_who: &T::AccountId) -> DispatchResult {
		unimplemented!()
	}
}

/// Handler for selecting the genesis validator set.
pub trait VerifyEthReceipts {
	fn verify_receipt(proof_record: &EthReceiptProof) -> Result<Receipt, DispatchError>;
}

impl<T: Trait> VerifyEthReceipts for Module<T> {
	/// confirm that the block hash is right
	/// get the receipt MPT trie root from the block header
	/// Using receipt MPT trie root to verify the proof and index etc.
	fn verify_receipt(proof_record: &EthReceiptProof) -> Result<Receipt, DispatchError> {
		let info = Self::header_info_of(&proof_record.header_hash).ok_or(<Error<T>>::HeaderInfoNE)?;

		let canonical_hash = Self::canonical_header_hash_of(info.number);
		ensure!(canonical_hash == proof_record.header_hash, <Error<T>>::HeaderNC);

		let best_info = Self::header_info_of(Self::best_header_hash()).ok_or(<Error<T>>::HeaderInfoNE)?;
		ensure!(
			best_info.number
				>= info
					.number
					.checked_add(Self::number_of_blocks_safe())
					.ok_or(<Error<T>>::BlockNumberOF)?,
			<Error<T>>::HeaderNS,
		);

		let header = Self::header_of(&proof_record.header_hash).ok_or(<Error<T>>::HeaderNE)?;
		let proof: Proof = rlp::decode(&proof_record.proof).map_err(|_| <Error<T>>::RlpDcF)?;
		let key = rlp::encode(&proof_record.index);
		let value = MerklePatriciaTrie::verify_proof(header.receipts_root().0.to_vec(), &key, proof)
			.map_err(|_| <Error<T>>::ProofVF)?
			.ok_or(<Error<T>>::TrieKeyNE)?;
		let receipt = rlp::decode(&value).map_err(|_| <Error<T>>::ReceiptDsF)?;

		Ok(receipt)
	}
}
