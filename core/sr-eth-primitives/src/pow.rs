/// A simplified prototype for light verification for pow.
use super::*;
//use crate::keccak::{keccak_256, keccak_512, H256 as BH256};
use core::cmp;
use core::convert::{From, Into, TryFrom};
use error::{BlockError, Mismatch, OutOfBounds};
use ethbloom::Bloom;
use keccak_hash::KECCAK_EMPTY_LIST_RLP;
use rstd::collections::btree_map::BTreeMap;
use rstd::mem;
use rstd::result;
use sr_primitives::RuntimeDebug;

use codec::{Decode, Encode};

use ethereum_types::BigEndianHash;
use primitive_types::{H256, U256, U512};

use rlp::*;

//use substrate_primitives::RuntimeDebug;

pub const MINIMUM_DIFFICULTY: u128 = 131072;
// TODO: please keep an eye on this.
// it might change due to ethereum's upgrade
pub const PROGPOW_TRANSITION: u64 = u64::max_value();
//pub const DIFFICULTY_HARDFORK_TRANSITION: u64 = 0x59d9;
pub const DIFFICULTY_HARDFORK_BOUND_DIVISOR: u128 = 0x0200;
pub const DIFFICULTY_BOUND_DIVISOR: u128 = 0x0800;
pub const EXPIP2_TRANSITION: u64 = 0xc3500;
pub const EXPIP2_DURATION_LIMIT: u64 = 0x1e;
pub const DURATION_LIMIT: u64 = 0x3C;
pub const HOMESTEAD_TRANSITION: u64 = 0x30d40;
pub const EIP100B_TRANSITION: u64 = 0xC3500;
pub const DIFFICULTY_INCREMENT_DIVISOR: u64 = 0x3C;
pub const METROPOLIS_DIFFICULTY_INCREMENT_DIVISOR: u64 = 0x1E;

pub const BOMB_DEFUSE_TRANSITION: u64 = 0x30d40;
// 3,000,000
pub const ECIP1010_PAUSE_TRANSITION: u64 = 0x2dc6c0;
// 5,000,000
pub const ECIP1010_CONTINUE_TRANSITION: u64 = 0x4c4b40;

pub const DIFFICULTY_HARDFORK_TRANSITION: u64 = u64::max_value();

#[derive(Default, PartialEq, Eq, Clone, Encode, Decode, RlpDecodable, RlpEncodable, RuntimeDebug)]
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

#[derive(PartialEq, Eq, Clone, Encode, Decode, Copy, RuntimeDebug)]
enum Seal {
	/// The seal/signature is included.
	With,
	/// The seal/signature is not included.
	Without,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
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
		self.hash.unwrap_or_else(|| keccak_hash::keccak(self.rlp(Seal::With)))
	}

	/// Get the hash of the header excluding the seal
	pub fn bare_hash(&self) -> H256 {
		keccak_hash::keccak(self.rlp(Seal::Without))
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

fn quick_get_difficulty(header_hash: &[u8; 32], nonce: u64, mix_hash: &[u8; 32], progpow: bool) -> [u8; 32] {
	let mut first_buf = [0u8; 40];
	let mut buf = [0u8; 64 + 32];

	let hash_len = header_hash.len();
	first_buf[..hash_len].copy_from_slice(header_hash);
	first_buf[hash_len..hash_len + mem::size_of::<u64>()].copy_from_slice(&nonce.to_ne_bytes());

	keccak_hash::keccak_512(&first_buf, &mut buf);
	buf[64..].copy_from_slice(mix_hash);

	let mut hash = [0u8; 32];
	keccak_hash::keccak_256(&buf, &mut hash);

	hash

	//	let mut buf = [0u8; 64 + 32];
	//
	//	#[cfg(feature = "std")]
	//	unsafe {
	//		let hash_len = header_hash.len();
	//		buf[..hash_len].copy_from_slice(header_hash);
	//		buf[hash_len..hash_len + mem::size_of::<u64>()].copy_from_slice(&nonce.to_ne_bytes());
	//
	//		keccak_512::unchecked(buf.as_mut_ptr(), 64, buf.as_ptr(), 40);
	//		buf[64..].copy_from_slice(mix_hash);
	//
	//		let mut hash = [0u8; 32];
	//		keccak_256::unchecked(hash.as_mut_ptr(), hash.len(), buf.as_ptr(), buf.len());
	//
	//		hash
	//	}
}

fn calculate_difficulty(header: &EthHeader, parent: &EthHeader) -> U256 {
	const EXP_DIFF_PERIOD: u64 = 100_000;

	let mut difficulty_bomb_delays = BTreeMap::<BlockNumber, BlockNumber>::new();
	difficulty_bomb_delays.insert(0xC3500, 3000000);
	if header.number() == 0 {
		panic!("Can't calculate genesis block difficulty");
	}

	let parent_has_uncles = parent.uncles_hash() != &KECCAK_EMPTY_LIST_RLP;

	let min_difficulty = U256::from(MINIMUM_DIFFICULTY);

	let difficulty_hardfork = header.number() >= DIFFICULTY_HARDFORK_TRANSITION;
	let difficulty_bound_divisor = U256::from(if difficulty_hardfork {
		DIFFICULTY_HARDFORK_BOUND_DIVISOR
	} else {
		DIFFICULTY_BOUND_DIVISOR
	});

	let expip2_hardfork = header.number() >= EXPIP2_TRANSITION;
	let duration_limit = if expip2_hardfork {
		EXPIP2_DURATION_LIMIT
	} else {
		DURATION_LIMIT
	};

	let frontier_limit = HOMESTEAD_TRANSITION;

	let mut target = if header.number() < frontier_limit {
		if header.timestamp() >= parent.timestamp() + duration_limit {
			*parent.difficulty() - (*parent.difficulty() / difficulty_bound_divisor)
		} else {
			*parent.difficulty() + (*parent.difficulty() / difficulty_bound_divisor)
		}
	} else {
		//		trace!(target: "ethash", "Calculating difficulty parent.difficulty={}, header.timestamp={}, parent.timestamp={}", parent.difficulty(), header.timestamp(), parent.timestamp());
		//block_diff = parent_diff + parent_diff // 2048 * max(1 - (block_timestamp - parent_timestamp) // 10, -99)
		let (increment_divisor, threshold) = if header.number() < EIP100B_TRANSITION {
			(DIFFICULTY_INCREMENT_DIVISOR, 1)
		} else if parent_has_uncles {
			(METROPOLIS_DIFFICULTY_INCREMENT_DIVISOR, 2)
		} else {
			(METROPOLIS_DIFFICULTY_INCREMENT_DIVISOR, 1)
		};

		let diff_inc = (header.timestamp() - parent.timestamp()) / increment_divisor;
		if diff_inc <= threshold {
			*parent.difficulty() + *parent.difficulty() / difficulty_bound_divisor * U256::from(threshold - diff_inc)
		} else {
			let multiplier: U256 = cmp::min(diff_inc - threshold, 99).into();
			parent
				.difficulty()
				.saturating_sub(*parent.difficulty() / difficulty_bound_divisor * multiplier)
		}
	};
	target = cmp::max(min_difficulty, target);
	if header.number() < BOMB_DEFUSE_TRANSITION {
		if header.number() < ECIP1010_PAUSE_TRANSITION {
			let mut number = header.number();
			let original_number = number;
			for (block, delay) in &difficulty_bomb_delays {
				if original_number >= *block {
					number = number.saturating_sub(*delay);
				}
			}
			let period = (number / EXP_DIFF_PERIOD) as usize;
			if period > 1 {
				target = cmp::max(min_difficulty, target + (U256::from(1) << (period - 2)));
			}
		} else if header.number() < ECIP1010_CONTINUE_TRANSITION {
			let fixed_difficulty = ((ECIP1010_PAUSE_TRANSITION / EXP_DIFF_PERIOD) - 2) as usize;
			target = cmp::max(min_difficulty, target + (U256::from(1) << fixed_difficulty));
		} else {
			let period = ((parent.number() + 1) / EXP_DIFF_PERIOD) as usize;
			let delay = ((ECIP1010_CONTINUE_TRANSITION - ECIP1010_PAUSE_TRANSITION) / EXP_DIFF_PERIOD) as usize;
			target = cmp::max(min_difficulty, target + (U256::from(1) << (period - delay - 2)));
		}
	}
	target
}

#[cfg(test)]
mod tests {
	use super::*;
	use error::BlockError;
	use hex_literal::*;
	use rustc_hex::FromHex;
	use std::str::FromStr;

	#[inline]
	fn sequential_header() -> (EthHeader, EthHeader) {
		let mixh1 = H256::from(hex!("543bc0769f7d5df30e7633f4a01552c2cee7baace8a6da37fddaa19e49e81209"));
		let nonce1 = H64::from(hex!("a5d3d0ccc8bb8a29"));
		// #8996777
		let header1 = EthHeader {
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
			seal: vec![rlp::encode(&mixh1), rlp::encode(&nonce1)],
			hash: Some(H256::from(hex!("b80bf91d6f459227a9c617c5d9823ff0b07f1098ea16788676f0b804ecd42f3b"))),
		};

		// # 8996778
		let mixh2 = H256::from(hex!("0ea8027f96c18f474e9bc74ff71d29aacd3f485d5825be0a8dde529eb82a47ed"));
		let nonce2 = H64::from(hex!("55859dc00728f99a"));
		let header2 = EthHeader {
			parent_hash: H256::from(hex!("b80bf91d6f459227a9c617c5d9823ff0b07f1098ea16788676f0b804ecd42f3b")),
			timestamp: 0x5ddb67a3,
			number: 0x8947aa,
			author: Address::from(hex!("d224ca0c819e8e97ba0136b3b95ceff503b79f53")),
			transactions_root: H256::from(hex!("efebac0e71cc2de04cf2f509bb038a82bbe92a659e010061b49b5387323b5ea6")),
			uncles_hash: H256::from(hex!("1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347")),
			extra_data: "7575706f6f6c2e636e2d3163613037623939".from_hex().unwrap(),
			state_root: H256::from(hex!("5dfc6357dda61a7f927292509afacd51453ff158342eb9628ccb419fbe91c638")),
			receipts_root: H256::from(hex!("3fbd99e253ff45045eec1e0011ac1b45fa0bccd641a356727defee3b166dd3bf")),
			log_bloom: Bloom::from_str("0c0110a00144a0082057622381231d842b8977a98d1029841000a1c21641d91946594605e902a5432000159ad24a0300428d8212bf4d1c81c0f8478402a4a818010011437c07a112080e9a4a14822311a6840436f26585c84cc0d50693c148bf9830cf3e0a08970788a4424824b009080d52372056460dec808041b68ea04050bf116c041f25a3329d281068740ca911c0d4cd7541a1539005521694951c286567942d0024852080268d29850000954188f25151d80e4900002122c01ad53b7396acd34209c24110b81b9278642024603cd45387812b0696d93992829090619cf0b065a201082280812020000430601100cb08a3808204571c0e564d828648fb").unwrap(),
			gas_used: 0x98254e.into(),
			gas_limit: 0x98700d.into(),
			difficulty: 0x92c07e50de0b9_u64.into(),
			seal: vec![rlp::encode(&mixh2), rlp::encode(&nonce2)],
			hash: Some(H256::from(hex!("b972df738904edb8adff9734eebdcb1d3b58fdfc68a48918720a4a247170f15e"))),
		};

		(header1, header2)
	}

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
	fn can_verify_basic_difficulty() {
		let header = sequential_header().0;
		assert_eq!(verify_block_basic(&header), Ok(()));
	}

	#[test]
	fn can_calculate_difficulty() {
		let (header1, header2) = sequential_header();
		let expected = U256::from_str("92c07e50de0b9").unwrap();
		assert_eq!(calculate_difficulty(&header2, &header1), expected);
	}
}
