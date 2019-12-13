/// A simplified prototype for light verification for pow.
use super::*;
//use crate::keccak::{keccak_256, keccak_512, H256 as BH256};
use codec::{Decode, Encode};
use core::cmp;
use core::convert::{From, Into, TryFrom};
use error::{BlockError, Mismatch, OutOfBounds};
use ethereum_types::BigEndianHash;
use header::EthHeader;
use keccak_hash::KECCAK_EMPTY_LIST_RLP;
use primitive_types::{H256, U256, U512};
use rlp::*;
use rstd::{collections::btree_map::BTreeMap, mem, result};
use sr_primitives::RuntimeDebug;

#[derive(Default, PartialEq, Eq, Clone, Encode, Decode)]
pub struct EthashPartial {
	pub minimum_difficulty: U256,
	pub difficulty_bound_divisor: U256,
	pub difficulty_increment_divisor: u64,
	pub metropolis_difficulty_increment_divisor: u64,
	pub duration_limit: u64,
	pub homestead_transition: u64,
	pub difficulty_hardfork_transition: u64,
	pub difficulty_hardfork_bound_divisor: U256,
	pub bomb_defuse_transition: u64,
	pub eip100b_transition: u64,
	pub ecip1010_pause_transition: u64,
	pub ecip1010_continue_transition: u64,
	pub difficulty_bomb_delays: BTreeMap<EthBlockNumber, EthBlockNumber>,
	pub expip2_transition: u64,
	pub expip2_duration_limit: u64,
	pub progpow_transition: u64,
}

impl EthashPartial {
	pub fn set_difficulty_bomb_delays(&mut self, key: EthBlockNumber, value: EthBlockNumber) {
		self.difficulty_bomb_delays.insert(key, value);
	}

	pub fn expanse() -> Self {
		EthashPartial {
			minimum_difficulty: U256::from(131072_u128),
			difficulty_bound_divisor: U256::from(0x0800),
			difficulty_increment_divisor: 0x3C,
			metropolis_difficulty_increment_divisor: 0x1E,
			duration_limit: 0x3C,
			homestead_transition: 0x30d40,
			difficulty_hardfork_transition: 0x59d9,
			difficulty_hardfork_bound_divisor: U256::from(0x0200),
			bomb_defuse_transition: 0x30d40,
			eip100b_transition: 0xC3500,
			ecip1010_pause_transition: 0x2dc6c0,
			ecip1010_continue_transition: 0x4c4b40,
			difficulty_bomb_delays: BTreeMap::<EthBlockNumber, EthBlockNumber>::default(),
			expip2_transition: 0xc3500,
			expip2_duration_limit: 0x1e,
			progpow_transition: u64::max_value(),
		}
	}

	pub fn production() -> Self {
		EthashPartial {
			minimum_difficulty: U256::from(131072_u128),
			difficulty_bound_divisor: U256::from(0x0800),
			difficulty_increment_divisor: 10,
			metropolis_difficulty_increment_divisor: 9,
			duration_limit: 13,
			homestead_transition: 1150000,
			difficulty_hardfork_transition: u64::max_value(),
			difficulty_hardfork_bound_divisor: U256::from(2048),
			bomb_defuse_transition: u64::max_value(),
			eip100b_transition: 4370000,
			ecip1010_pause_transition: u64::max_value(),
			ecip1010_continue_transition: u64::max_value(),
			difficulty_bomb_delays: {
				let mut m = BTreeMap::new();
				m.insert(4370000, 3000000);
				m.insert(7280000, 2000000);
				m
			},
			expip2_transition: u64::max_value(),
			expip2_duration_limit: 30,
			progpow_transition: u64::max_value(),
		}
	}

	pub fn ropsten_testnet() -> Self {
		EthashPartial {
			minimum_difficulty: U256::from(0x20000),
			difficulty_bound_divisor: U256::from(0x0800),
			difficulty_increment_divisor: 10,
			metropolis_difficulty_increment_divisor: 9,
			duration_limit: 0xd,
			homestead_transition: 0x0,
			difficulty_hardfork_transition: 0x59d9,
			difficulty_hardfork_bound_divisor: U256::from(0x0800),
			bomb_defuse_transition: u64::max_value(),
			eip100b_transition: 0x19f0a0,
			ecip1010_pause_transition: u64::max_value(),
			ecip1010_continue_transition: u64::max_value(),
			difficulty_bomb_delays: {
				let mut m = BTreeMap::new();
				m.insert(0x19f0a0, 0x2dc6c0);
				m.insert(0x408b70, 0x1e8480);
				m
			},
			expip2_transition: u64::max_value(),
			expip2_duration_limit: 30,
			progpow_transition: u64::max_value(),
		}
	}
}

impl EthashPartial {
	pub fn verify_block_basic(&self, header: &EthHeader) -> result::Result<(), error::BlockError> {
		// check the seal fields.
		let seal = EthashSeal::parse_seal(header.seal())?;

		// TODO: consider removing these lines.
		let min_difficulty = self.minimum_difficulty;
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
			header.number() >= self.progpow_transition,
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

	pub fn calculate_difficulty(&self, header: &EthHeader, parent: &EthHeader) -> U256 {
		const EXP_DIFF_PERIOD: u64 = 100_000;

		if header.number() == 0 {
			panic!("Can't calculate genesis block difficulty");
		}

		let parent_has_uncles = parent.uncles_hash() != &KECCAK_EMPTY_LIST_RLP;

		let min_difficulty = self.minimum_difficulty;

		let difficulty_hardfork = header.number() >= self.difficulty_hardfork_transition;
		let difficulty_bound_divisor = if difficulty_hardfork {
			self.difficulty_hardfork_bound_divisor
		} else {
			self.difficulty_bound_divisor
		};

		let expip2_hardfork = header.number() >= self.expip2_transition;
		let duration_limit = if expip2_hardfork {
			self.expip2_duration_limit
		} else {
			self.duration_limit
		};

		let frontier_limit = self.homestead_transition;

		let mut target = if header.number() < frontier_limit {
			if header.timestamp() >= parent.timestamp() + duration_limit {
				*parent.difficulty() - (*parent.difficulty() / difficulty_bound_divisor)
			} else {
				*parent.difficulty() + (*parent.difficulty() / difficulty_bound_divisor)
			}
		} else {
			//		trace!(target: "ethash", "Calculating difficulty parent.difficulty={}, header.timestamp={}, parent.timestamp={}", parent.difficulty(), header.timestamp(), parent.timestamp());
			//block_diff = parent_diff + parent_diff // 2048 * max(1 - (block_timestamp - parent_timestamp) // 10, -99)
			let (increment_divisor, threshold) = if header.number() < self.eip100b_transition {
				(self.difficulty_increment_divisor, 1)
			} else if parent_has_uncles {
				(self.metropolis_difficulty_increment_divisor, 2)
			} else {
				(self.metropolis_difficulty_increment_divisor, 1)
			};

			let diff_inc = (header.timestamp() - parent.timestamp()) / increment_divisor;
			if diff_inc <= threshold {
				*parent.difficulty()
					+ *parent.difficulty() / difficulty_bound_divisor * U256::from(threshold - diff_inc)
			} else {
				let multiplier: U256 = cmp::min(diff_inc - threshold, 99).into();
				parent
					.difficulty()
					.saturating_sub(*parent.difficulty() / difficulty_bound_divisor * multiplier)
			}
		};
		target = cmp::max(min_difficulty, target);
		if header.number() < self.bomb_defuse_transition {
			if header.number() < self.ecip1010_pause_transition {
				let mut number = header.number();
				let original_number = number;
				for (block, delay) in &self.difficulty_bomb_delays {
					if original_number >= *block {
						number = number.saturating_sub(*delay);
					}
				}
				let period = (number / EXP_DIFF_PERIOD) as usize;
				if period > 1 {
					target = cmp::max(min_difficulty, target + (U256::from(1) << (period - 2)));
				}
			} else if header.number() < self.ecip1010_continue_transition {
				let fixed_difficulty = ((self.ecip1010_pause_transition / EXP_DIFF_PERIOD) - 2) as usize;
				target = cmp::max(min_difficulty, target + (U256::from(1) << fixed_difficulty));
			} else {
				let period = ((parent.number() + 1) / EXP_DIFF_PERIOD) as usize;
				let delay =
					((self.ecip1010_continue_transition - self.ecip1010_pause_transition) / EXP_DIFF_PERIOD) as usize;
				target = cmp::max(min_difficulty, target + (U256::from(1) << (period - delay - 2)));
			}
		}
		target
	}
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
			.map_err(|_e| BlockError::Rlp("Rlp - INVALID"))
			.unwrap();
		let nonce = Rlp::new(seal[1].as_ref())
			.as_val::<H64>()
			.map_err(|_e| BlockError::Rlp("Rlp - INVALID"))
			.unwrap();
		Ok(EthashSeal { mix_hash, nonce })
	}
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

fn quick_get_difficulty(header_hash: &[u8; 32], nonce: u64, mix_hash: &[u8; 32], _progpow: bool) -> [u8; 32] {
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
