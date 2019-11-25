/// A simplified prototype for light verification for pow.
use super::*;
use crate::keccak::{keccak_256, keccak_512, H256 as BH256};
use core::convert::{From, Into, TryFrom};
use error::{BlockError, Mismatch, OutOfBounds};
use rstd::mem;
use rstd::result;

pub const MINIMUM_DIFFICULTY: u128 = 131072;
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

	/// Set the difficulty field of the header.
	pub fn set_difficulty(&mut self, a: U256) {
		change_field(&mut self.hash, &mut self.difficulty, a);
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
	let min_difficulty = MINIMUM_DIFFICULTY.into();
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

fn quick_get_difficulty(header_hash: &BH256, nonce: u64, mix_hash: &BH256, progpow: bool) -> BH256 {
	let mut buf = [0u8; 64 + 32];

	#[cfg(feature = "std")]
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
	use hex_literal::*;
	use rustc_hex::FromHex;
	use std::str::FromStr;

	#[test]
	fn can_do_proof_of_work_verification_fail() {
		let mut header: EthHeader = EthHeader::default();
		header.set_seal(vec![rlp::encode(&H256::zero()), rlp::encode(&H64::zero())]);
		header.set_difficulty(
			U256::from_str("ffffffffffffffffffffffffffffffffffffffffffffaaaaaaaaaaaaaaaaaaaa").unwrap(),
		);

		let verify_result = verify_block_basic(&header);

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

	#[test]
	fn can_verify_mainnet_difficulty() {
		let mix_hash = H256::from(hex!("543bc0769f7d5df30e7633f4a01552c2cee7baace8a6da37fddaa19e49e81209"));
		let nonce = H64::from(hex!("a5d3d0ccc8bb8a29"));
		let header = EthHeader {
			parent_hash: H256::from(hex!("0b2d720b8d3b6601e4207ef926b0c228735aa1d58301a23d58f9cb51ac2288d8")),
			timestamp: 0x5ddb67a0,
			number: 0x8947a9,
			author: Address::from(hex!("4c549990a7ef3fea8784406c1eecc98bf4211fa5")),
			transactions_root: H256::from(hex!("07d44fadb4aca78c81698710211c5399c1408bb3f0aa3a687d091d230fcaddc6")),
			uncles_hash: H256::from(hex!("1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347")),
			extra_data: "5050594520686976656f6e2d6574682d6672".from_hex().unwrap(),
			state_root: H256::from(hex!("4ba0fb3e6f4c1af32a799df667d304bcdb7f8154e6f86831f92f5a354c2baf70")),
			receipts_root: H256::from(hex!("5968afe6026e673df3b9745d925a5648282d2195a46c22771fec48210daf8e23")),
			log_bloom: Bloom::from_str("0c7b091bc8ec02401ad12491004e3014e8806390031950181c118580ac61c9a00409022c418162002710a991108a11ca5383d4921d1da46346edc3eb8068481118b005c0b20700414c13916c54011a0922904aa6e255406a33494c84a1426410541819070e04852042410b30030d4c88a5103082284c7d9bd42090322ae883e004224e18db4d858a0805d043e44a855400945311cb253001412002ea041a08e30394fc601440310920af2192dc4194a03302191cf2290ac0c12000815324eb96a08000aad914034c1c8eb0cb39422e272808b7a4911989c306381502868820b4b95076fc004b14dd48a0411024218051204d902b80d004c36510400ccb123084").unwrap(),
			gas_used: 0x986d77.into(),
			gas_limit: 0x989631.into(),
			difficulty: 0x92ac28cbc4930_u64.into(),
			seal: vec![rlp::encode(&mix_hash), rlp::encode(&nonce)],
			hash: Some(H256::from(hex!("b80bf91d6f459227a9c617c5d9823ff0b07f1098ea16788676f0b804ecd42f3b"))),
		};

		assert_eq!(verify_block_basic(&header), Ok(()));
	}

}
