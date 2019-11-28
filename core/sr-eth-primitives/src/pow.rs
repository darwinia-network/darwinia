/// A simplified prototype for light verification for pow.
use super::*;
//use crate::keccak::{keccak_256, keccak_512, H256 as BH256};
use codec::{Decode, Encode};
use core::cmp;
use core::convert::{From, Into, TryFrom};
use error::{BlockError, Mismatch, OutOfBounds};
use ethbloom::Bloom;
use header::EthHeader;
use keccak_hash::KECCAK_EMPTY_LIST_RLP;
use rstd::collections::btree_map::BTreeMap;
use rstd::mem;
use rstd::result;

use ethereum_types::BigEndianHash;
use primitive_types::{H160, H256, U128, U256, U512};

use rlp::*;

//use substrate_primitives::RuntimeDebug;

pub const MINIMUM_DIFFICULTY: u128 = 131072;
// TODO: please keep an eye on this.
// it might change due to ethereum's upgrade
pub const PROGPOW_TRANSITION: u64 = u64::max_value();
pub const DIFFICULTY_HARDFORK_TRANSITION: u64 = 0x59d9;
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

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
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

pub fn calculate_difficulty(header: &EthHeader, parent: &EthHeader) -> U256 {
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
