//! utils for parsing eth headers
use std::num::ParseIntError;
use std::str::FromStr;

use crate::*;

pub use eth_primitives::{receipt::LogEntry, Bloom, EthAddress, H64};
pub use json::{object::Object, JsonValue};

/// mock ethheader
pub fn mock_header_from_source<'m>(o: &'m Object) -> Option<EthHeader> {
	let mixh = H256::from(bytes!(&o.get("mixHash")?, 32));
	let nonce = H64::from(bytes!(&o.get("nonce")?, 8));

	Some(EthHeader {
		parent_hash: H256::from(bytes!(&o.get("parentHash")?, 32)),
		timestamp: o.get("timestamp")?.as_u64()?,
		number: o.get("number")?.as_u64()?,
		author: EthAddress::from(bytes!(&o.get("miner")?, 20)),
		transactions_root: H256::from(bytes!(&o.get("transactionsRoot")?, 32)),
		uncles_hash: H256::from(bytes!(&o.get("sha3Uncles")?, 32)),
		extra_data: hex(&o.get("extraData")?.as_str()?)?,
		state_root: H256::from(bytes!(&o.get("stateRoot")?, 32)),
		receipts_root: H256::from(bytes!(&o.get("receiptsRoot")?, 32)),
		log_bloom: Bloom::from_str(&o.get("logsBloom")?.as_str()?[2..]).unwrap(),
		gas_used: o.get("gasUsed")?.as_i32()?.into(),
		gas_limit: o.get("gasLimit")?.as_i32()?.into(),
		difficulty: o.get("difficulty")?.as_str()?.parse::<u64>().unwrap_or(0).into(),
		seal: vec![rlp::encode(&mixh), rlp::encode(&nonce)],
		hash: Some(H256::from(bytes!(&o.get("hash")?, 32))),
	})
}

/// mock receipt
pub fn mock_receipt_from_source(o: &mut Object) -> Option<EthReceiptProof> {
	Some(EthReceiptProof {
		index: o.get("index")?.as_str()?[2..].parse::<u64>().unwrap(),
		proof: hex(&o.get("proof")?.as_str()?)?,
		header_hash: H256::from(bytes!(&o.get("header_hash")?, 32)),
	})
}

/// mock logs
pub fn mock_log_from_source(o: &mut Object) -> Option<LogEntry> {
	let mut topics: Vec<H256> = vec![];
	if let JsonValue::Array(ts) = o.remove("topics")? {
		for t in ts.iter() {
			topics.push(H256::from(bytes!(t, 32)));
		}
	}

	Some(LogEntry {
		address: EthAddress::from(bytes!(&o.get("address")?, 20)),
		topics,
		data: hex(&o.get("data")?.as_str()?)?,
	})
}

/// convert hex string to byte array
fn hex(s: &str) -> Option<Vec<u8>> {
	let r: Result<Vec<u8>, ParseIntError> = (2..s.len())
		.step_by(2)
		.map(|i| u8::from_str_radix(&s[i..i + 2], 16))
		.collect();

	if let Ok(r) = r {
		Some(r)
	} else {
		None
	}
}

/// readable macro
#[macro_export]
macro_rules! bytes {
	($str:expr, $len: tt) => {{
		let mut bytes: [u8; $len] = [0; $len];
		bytes.copy_from_slice(&hex(&$str.as_str()?)?);
		bytes
		}};
}
