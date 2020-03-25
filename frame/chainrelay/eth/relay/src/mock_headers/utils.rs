//! utils for parsing eth headers
use std::{fmt::Debug, str::FromStr};

use eth_primitives::{Bloom, EthAddress, H64};
use hex::FromHex as HexFromHex;
use json::object::Object;
use rustc_hex::FromHex;

use crate::*;

pub use eth_primitives::receipt::LogEntry;
pub use json::JsonValue;

/// get hash
fn get_hash(o: &mut Object, key: &str) -> Option<H256> {
	Some(H256::from(<[u8; 32]>::from_hex(&o.remove(key)?.as_str()?[2..]).ro()?))
}

/// convert Result to Option
///
/// NOTE: panic should happen in tests, but not mock.
trait ResultToOption<T> {
	fn ro(self) -> Option<T>;
}

impl<T, E> ResultToOption<T> for Result<T, E>
where
	E: Debug,
{
	fn ro(self) -> Option<T> {
		match self {
			Ok(v) => Some(v),
			Err(_) => None,
		}
	}
}

/// mock ethheader
pub fn mock_header_from_source<'m>(o: &'m mut Object) -> Option<EthHeader> {
	let mixh = get_hash(o, "mixHash")?;
	let nonce = H64::from(<[u8; 8]>::from_hex(&o.remove("nonce")?.as_str()?[2..]).ro()?);
	let addr = <[u8; 20]>::from_hex(&o.remove("miner")?.as_str()?[2..]).ro()?;

	let extra_data = o.remove("extraData")?;
	let eds = &extra_data.as_str()?[2..];

	let log_bloom = o.remove("logsBloom")?;
	let lb = &log_bloom.as_str()?[2..];

	let difficulty: u64 = o.remove("difficulty")?.as_str()?.parse().ro()?;

	Some(EthHeader {
		parent_hash: get_hash(o, "parentHash")?,
		timestamp: o.remove("timestamp")?.as_u64()?,
		number: o.remove("number")?.as_u64()?,
		author: EthAddress::from(addr),
		transactions_root: get_hash(o, "transactionsRoot")?,
		uncles_hash: get_hash(o, "sha3Uncles")?,
		extra_data: eds.from_hex().unwrap(),
		state_root: get_hash(o, "stateRoot")?,
		receipts_root: get_hash(o, "receiptsRoot")?,
		log_bloom: Bloom::from_str(lb).unwrap(),
		gas_used: o.remove("gasUsed")?.as_i32()?.into(),
		gas_limit: o.remove("gasLimit")?.as_i32()?.into(),
		difficulty: difficulty.into(),
		seal: vec![rlp::encode(&mixh), rlp::encode(&nonce)],
		hash: Some(get_hash(o, "hash")?),
	})
}

/// mock receipt
pub fn mock_receipt_from_source(o: &mut Object) -> Option<EthReceiptProof> {
	let index: u64 = {
		let i = o.remove("index")?;
		if let Ok(idx) = i.as_str()?[2..].parse() {
			idx
		} else {
			return None;
		}
	};

	let proof: Vec<u8> = {
		let p = o.remove("proof")?;
		if let Ok(prf) = p.as_str()?[2..].from_hex() {
			prf
		} else {
			return None;
		}
	};

	Some(EthReceiptProof {
		index,
		proof,
		header_hash: get_hash(o, "header_hash")?,
	})
}

/// mock logs
pub fn mock_log_from_source(o: &mut Object) -> Option<LogEntry> {
	let data: Vec<u8> = {
		let p = o.remove("data")?;
		if let Ok(d) = p.as_str()?[2..].from_hex() {
			d
		} else {
			return None;
		}
	};

	let mut topics: Vec<H256> = vec![];
	if let JsonValue::Array(ts) = o.remove("topics")? {
		for t in ts.iter() {
			topics.push(H256::from(<[u8; 32]>::from_hex(&t.as_str()?[2..]).ro()?));
		}
	}

	Some(LogEntry {
		address: EthAddress::from(<[u8; 20]>::from_hex(&o.remove("address")?.as_str()?[2..]).ro()?),
		topics,
		data,
	})
}
