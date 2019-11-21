/// A simplified prototype for light verification for pow.
use super::*;
use crate::keccak::{keccak_256, keccak_512};
use core::convert::TryFrom;
use error::{BlockError, Mismatch, OutOfBounds};
use rstd::mem;
use rstd::result;

pub const MINIMUM_DIFFICULTY: U256 = 0x20000.into();
// TODO: please keep an eye on this.
// it might change due to ethereum's upgrade
pub const PROGPOW_TRANSITION: u64 = u64::max_value();

#[derive(Default, PartialEq, Eq, Clone)]
pub struct EthHeader {
	parent_hash: H256,
	timestamp: u64,
	number: BlockNumber,
	author: Address,
	transactions_root: H256,
	uncles_hash: H256,
	extra_data: Bytes,
	state_root: H256,
	receipts_root: H256,
	log_bloom: Bloom,
	gas_used: U256,
	gas_limit: U256,
	difficulty: U256,
	seal: Vec<Bytes>,
	hash: Option<H256>,
}
/// Alter value of given field, reset memoised hash if changed.
fn change_field<T>(hash: &mut Option<H256>, field: &mut T, value: T)
where
	T: PartialEq<T>,
{
	if field != &value {
		*field = value;
		*hash = None;
	}
}

#[derive(Clone, Copy)]
enum Seal {
	/// The seal/signature is included.
	With,
	/// The seal/signature is not included.
	Without,
}

#[derive(PartialEq)]
pub struct EthashSeal {
	/// Ethash seal mix_hash
	pub mix_hash: H256,
	/// Ethash seal nonce
	pub nonce: H64,
}

impl EthashSeal {
	/// Tries to parse rlp encoded bytes as an Ethash/Clique seal.
	pub fn parse_seal<T: AsRef<[u8]>>(seal: &[T]) -> Result<Self, BlockError> {
		if seal.len() != 2 {
			return Err(BlockError::InvalidSealArity(Mismatch {
				expected: 2,
				found: seal.len(),
			})
			.into());
		}

		let mix_hash = Rlp::new(seal[0].as_ref())
			.as_val::<H256>()
			.map_err(|_e| BlockError::Rlp("wrong rlp"))
			.unwrap();
		let nonce = Rlp::new(seal[1].as_ref())
			.as_val::<H64>()
			.map_err(|_e| BlockError::Rlp("wrong rlp"))
			.unwrap();
		Ok(EthashSeal { mix_hash, nonce })
	}
}

impl EthHeader {
	/// Create a new, default-valued, header.
	pub fn new() -> Self {
		Self::default()
	}

	/// Get the parent_hash field of the header.
	pub fn parent_hash(&self) -> &H256 {
		&self.parent_hash
	}

	/// Get the timestamp field of the header.
	pub fn timestamp(&self) -> u64 {
		self.timestamp
	}

	/// Get the number field of the header.
	pub fn number(&self) -> BlockNumber {
		self.number
	}

	/// Get the author field of the header.
	pub fn author(&self) -> &Address {
		&self.author
	}

	/// Get the extra data field of the header.
	pub fn extra_data(&self) -> &Bytes {
		&self.extra_data
	}

	/// Get the state root field of the header.
	pub fn state_root(&self) -> &H256 {
		&self.state_root
	}

	/// Get the receipts root field of the header.
	pub fn receipts_root(&self) -> &H256 {
		&self.receipts_root
	}

	/// Get the log bloom field of the header.
	pub fn log_bloom(&self) -> &Bloom {
		&self.log_bloom
	}

	/// Get the transactions root field of the header.
	pub fn transactions_root(&self) -> &H256 {
		&self.transactions_root
	}

	/// Get the uncles hash field of the header.
	pub fn uncles_hash(&self) -> &H256 {
		&self.uncles_hash
	}

	/// Get the gas used field of the header.
	pub fn gas_used(&self) -> &U256 {
		&self.gas_used
	}

	/// Get the gas limit field of the header.
	pub fn gas_limit(&self) -> &U256 {
		&self.gas_limit
	}

	/// Get the difficulty field of the header.
	pub fn difficulty(&self) -> &U256 {
		&self.difficulty
	}

	/// Get the seal field of the header.
	pub fn seal(&self) -> &[Bytes] {
		&self.seal
	}

	/// Set the seal field of the header.
	pub fn set_seal(&mut self, a: Vec<Bytes>) {
		change_field(&mut self.hash, &mut self.seal, a)
	}

	/// Get & memoize the hash of this header (keccak of the RLP with seal).
	pub fn compute_hash(&mut self) -> H256 {
		let hash = self.hash();
		self.hash = Some(hash);
		hash
	}

	/// Get the hash of this header (keccak of the RLP with seal).
	pub fn hash(&self) -> H256 {
		self.hash.unwrap_or_else(|| keccak(self.rlp(Seal::With)))
	}

	/// Get the hash of the header excluding the seal
	pub fn bare_hash(&self) -> H256 {
		keccak(self.rlp(Seal::Without))
	}

	/// Encode the header, getting a type-safe wrapper around the RLP.
	pub fn encoded(&self) -> encoded::Header {
		encoded::Header::new(self.rlp(Seal::With))
	}

	/// Get the RLP representation of this Header.
	fn rlp(&self, with_seal: Seal) -> Bytes {
		let mut s = RlpStream::new();
		self.stream_rlp(&mut s, with_seal);
		s.out()
	}

	/// Place this header into an RLP stream `s`, optionally `with_seal`.
	fn stream_rlp(&self, s: &mut RlpStream, with_seal: Seal) {
		if let Seal::With = with_seal {
			s.begin_list(13 + self.seal.len());
		} else {
			s.begin_list(13);
		}

		s.append(&self.parent_hash);
		s.append(&self.uncles_hash);
		s.append(&self.author);
		s.append(&self.state_root);
		s.append(&self.transactions_root);
		s.append(&self.receipts_root);
		s.append(&self.log_bloom);
		s.append(&self.difficulty);
		s.append(&self.number);
		s.append(&self.gas_limit);
		s.append(&self.gas_used);
		s.append(&self.timestamp);
		s.append(&self.extra_data);

		if let Seal::With = with_seal {
			for b in &self.seal {
				s.append_raw(b, 1);
			}
		}
	}
}

pub fn verify_block_basic(header: &EthHeader) -> result::Result<(), error::BlockError> {
	// check the seal fields.
	let seal = EthashSeal::parse_seal(header.seal())?;

	// TODO: consider removing these lines.
	let min_difficulty = MINIMUM_DIFFICULTY;
	if header.difficulty() < &min_difficulty {
		return Err(BlockError::DifficultyOutOfBounds(OutOfBounds {
			min: Some(min_difficulty),
			max: None,
			found: header.difficulty().clone(),
		}));
	}

	let difficulty = boundary_to_difficulty(&H256(quick_get_difficulty(
		&header.bare_hash().0,
		seal.nonce.to_low_u64_be(),
		&seal.mix_hash.0,
		header.number() >= PROGPOW_TRANSITION,
	)));

	if &difficulty < header.difficulty() {
		return Err(BlockError::InvalidProofOfWork(OutOfBounds {
			min: Some(header.difficulty().clone()),
			max: None,
			found: difficulty,
		}));
	}

	Ok(())
}

pub fn boundary_to_difficulty(boundary: &ethereum_types::H256) -> U256 {
	difficulty_to_boundary_aux(&boundary.into_uint())
}

fn difficulty_to_boundary_aux<T: Into<U512>>(difficulty: T) -> ethereum_types::U256 {
	let difficulty = difficulty.into();

	assert!(!difficulty.is_zero());

	if difficulty == U512::one() {
		U256::max_value()
	} else {
		const PROOF: &str = "difficulty > 1, so result never overflows 256 bits; qed";
		U256::try_from((U512::one() << 256) / difficulty).expect(PROOF)
	}
}

fn quick_get_difficulty(header_hash: &H256, nonce: u64, mix_hash: &H256, progpow: bool) -> H256 {
	let mut buf = [0u8; 64 + 32];

	unsafe {
		let hash_len = header_hash.len();
		buf[..hash_len].copy_from_slice(header_hash);
		buf[hash_len..hash_len + mem::size_of::<u64>()].copy_from_slice(&nonce.to_ne_bytes());

		keccak_512::unchecked(buf.as_mut_ptr(), 64, buf.as_ptr(), 40);
		buf[64..].copy_from_slice(mix_hash);

		let mut hash = [0u8; 32];
		keccak_256::unchecked(hash.as_mut_ptr(), hash.len(), buf.as_ptr(), buf.len());

		hash
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use error::BlockError;
	use eth_spec as spec;
	use std::str::FromStr;

	#[test]
	fn can_do_proof_of_work_verification_fail() {
		let engine = test_spec().engine;
		let mut header: EthHeader = EthHeader::default();
		header.set_seal(vec![rlp::encode(&H256::zero()), rlp::encode(&H64::zero())]);
		header.set_difficulty(
			U256::from_str("ffffffffffffffffffffffffffffffffffffffffffffaaaaaaaaaaaaaaaaaaaa").unwrap(),
		);

		let verify_result = engine.verify_block_basic(&header);

		match verify_result {
			Err(BlockError::InvalidProofOfWork(_)) => {}
			Err(_) => {
				panic!("should be invalid proof of work error (got {:?})", verify_result);
			}
			_ => {
				panic!("Should be error, got Ok");
			}
		}
	}

}
