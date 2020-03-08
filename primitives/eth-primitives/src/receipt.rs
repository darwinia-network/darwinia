use codec::{Decode, Encode};
use ethbloom::{Bloom, Input as BloomInput};
use primitive_types::{H256, U256};
use rlp::*;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

use crate::*;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum TransactionOutcome {
	/// Status and state root are unknown under EIP-98 rules.
	Unknown,
	/// State root is known. Pre EIP-98 and EIP-658 rules.
	StateRoot(H256),
	/// Status code is known. EIP-658 rules.
	StatusCode(u8),
}

#[derive(PartialEq, Eq, Clone, RlpEncodable, RlpDecodable, Encode, Decode, RuntimeDebug)]
pub struct LogEntry {
	/// The address of the contract executing at the point of the `LOG` operation.
	pub address: EthAddress,
	/// The topics associated with the `LOG` operation.
	pub topics: Vec<H256>,
	/// The data associated with the `LOG` operation.
	pub data: Bytes,
}

impl LogEntry {
	/// Calculates the bloom of this log entry.
	pub fn bloom(&self) -> Bloom {
		self.topics
			.iter()
			.fold(Bloom::from(BloomInput::Raw(self.address.as_bytes())), |mut b, t| {
				b.accrue(BloomInput::Raw(t.as_bytes()));
				b
			})
	}
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct Receipt {
	/// The total gas used in the block following execution of the transaction.
	pub gas_used: U256,
	/// The OR-wide combination of all logs' blooms for this transaction.
	pub log_bloom: Bloom,
	/// The logs stemming from this transaction.
	pub logs: Vec<LogEntry>,
	/// Transaction outcome.
	pub outcome: TransactionOutcome,
}

impl Receipt {
	/// Create a new receipt.
	pub fn new(outcome: TransactionOutcome, gas_used: U256, logs: Vec<LogEntry>) -> Self {
		Self {
			gas_used,
			log_bloom: logs.iter().fold(Bloom::default(), |mut b, l| {
				b.accrue_bloom(&l.bloom());
				b
			}),
			logs,
			outcome,
		}
	}
}

impl Encodable for Receipt {
	fn rlp_append(&self, s: &mut RlpStream) {
		match self.outcome {
			TransactionOutcome::Unknown => {
				s.begin_list(3);
			}
			TransactionOutcome::StateRoot(ref root) => {
				s.begin_list(4);
				s.append(root);
			}
			TransactionOutcome::StatusCode(ref status_code) => {
				s.begin_list(4);
				s.append(status_code);
			}
		}
		s.append(&self.gas_used);
		s.append(&self.log_bloom);
		s.append_list(&self.logs);
	}
}

impl Decodable for Receipt {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		if rlp.item_count()? == 3 {
			Ok(Receipt {
				outcome: TransactionOutcome::Unknown,
				gas_used: rlp.val_at(0)?,
				log_bloom: rlp.val_at(1)?,
				logs: rlp.list_at(2)?,
			})
		} else {
			Ok(Receipt {
				gas_used: rlp.val_at(1)?,
				log_bloom: rlp.val_at(2)?,
				logs: rlp.list_at(3)?,
				outcome: {
					let first = rlp.at(0)?;
					if first.is_data() && first.data()?.len() <= 1 {
						TransactionOutcome::StatusCode(first.as_val()?)
					} else {
						TransactionOutcome::StateRoot(first.as_val()?)
					}
				},
			})
		}
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use hex_literal::*;
	use keccak_hasher::KeccakHasher;
	use rustc_hex::FromHex;

	use super::*;

	#[inline]
	fn construct_receipts(
		root: Option<H256>,
		gas_used: U256,
		status: Option<u8>,
		log_entries: Vec<LogEntry>,
	) -> Receipt {
		Receipt::new(
			if root.is_some() {
				TransactionOutcome::StateRoot(root.unwrap())
			} else {
				TransactionOutcome::StatusCode(status.unwrap())
			},
			gas_used,
			log_entries,
		)
	}

	#[test]
	/// ropsten tx hash: 0xce62c3d1d2a43cfcc39707b98de53e61a7ef7b7f8853e943d85e511b3451aa7e
	fn test_basic() {
		// https://ropsten.etherscan.io/tx/0xce62c3d1d2a43cfcc39707b98de53e61a7ef7b7f8853e943d85e511b3451aa7e#eventlog
		let log_entries = vec![LogEntry {
			address: EthAddress::from_str("ad52e0f67b6f44cd5b9a6f4fbc7c0f78f37e094b").unwrap(),
			topics: vec![
				H256::from(hex!("6775ce244ff81f0a82f87d6fd2cf885affb38416e3a04355f713c6f008dd126a")),
				H256::from(hex!("0000000000000000000000000000000000000000000000000000000000000006")),
				H256::from(hex!("0000000000000000000000000000000000000000000000000000000000000000")),
			],
			data: "00000000000000000000000074241db5f3ebaeecf9506e4ae9881860933416048eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48000000000000000000000000000000000000000000000000002386f26fc10000".from_hex().unwrap(),
		}];

		//		let receipt = Receipt::new(
		//			TransactionOutcome::StatusCode(1),
		//			//				TransactionOutcome::StateRoot(H256::from(hex!("a21cdf375ebef58f606c298d6211f4edee58f2dd6430edbdd0ed3cd886a16863"))),
		//			U256::from(U128::from(1123401)),
		//			log_entries,
		//		);

		let r = construct_receipts(None, U256::from(U128::from(1123401)), Some(1), log_entries);
		//		let rs = &rlp::encode(&r)[..];
		// TODO: Check the log bloom generation logic
		assert_eq!(r.log_bloom, Bloom::from_str(
			"00000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000000000000000820000000000000020000000000000000000800000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000200000000020000000000000000000000000000080000000000000800000000000000000000000"
		).unwrap());
	}

	#[test]
	/// kovan tx hash: 0xaaf52845694258509cbdd30ea21894b4e685eb4cdbb13dd298f925fe6e51db35
	/// block number: 13376543 (only a tx in this block, which is above)
	/// from: 0x4aea6cfc5bd14f2308954d544e1dc905268357db
	/// to: 0xa24df0420de1f3b8d740a52aaeb9d55d6d64478e (a contract)
	/// receipts_root in block#13376543: 0xc789eb8b7f5876f4df4f8ae16f95c9881eabfb700ee7d8a00a51fb4a71afbac9
	/// to check if receipts_root in block-header can be pre-computed.
	fn check_receipts() {
		let expected_root = H256::from(hex!("c789eb8b7f5876f4df4f8ae16f95c9881eabfb700ee7d8a00a51fb4a71afbac9"));
		let log_entries = vec![LogEntry {
			address: EthAddress::from_str("a24df0420de1f3b8d740a52aaeb9d55d6d64478e").unwrap(),
			topics: vec![H256::from(hex!("f36406321d51f9ba55d04e900c1d56caac28601524e09d53e9010e03f83d7d00"))],
			data: "0000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000363384a3868b9000000000000000000000000000000000000000000000000000000005d75f54f0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000e53504f5450582f4241542d455448000000000000000000000000000000000000".from_hex().unwrap(),
		}];
		let receipts = vec![Receipt::new(
			TransactionOutcome::StatusCode(1),
			U256::from(U128::from(73705)),
			log_entries,
		)];

		let receipts_root: H256 = H256(triehash::ordered_trie_root::<KeccakHasher, _>(
			receipts.iter().map(|x| ::rlp::encode(x)),
		));

		//		let receipts_root: H256 = triehash_ethereum::ordered_trie_root<KeccakHasher, _>();

		assert_eq!(receipts_root, expected_root);
	}
}
