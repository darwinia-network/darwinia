//! utils for parsing eth headers
use std::num::ParseIntError;
use std::str::FromStr;

use eth_primitives::{Bloom, EthAddress, H64};
use hex_literal::hex;
use json::object::Object;
use rustc_hex::FromHex;

use crate::*;

/// hex
macro_rules! hex {
    {$(($name:tt, $len:tt),)*} => {
        $(
            pub fn $name(s: &str) -> Option<[u8; $len]> {
	            let r: Result<Vec<u8>, ParseIntError> = (2..s.len())
		            .step_by(2)
		            .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
		            .collect();

	            if let Ok(r) = r {
		            let mut bytes: [u8; $len] = [0; $len];
		            bytes.copy_from_slice(&r);

		            Some(bytes)
	            } else {
		            None
	            }
            }
        )*
    };

}

hex! {
	(h32, 32),
	(h20, 20),
	(h8, 8),
}

/// get_hash
pub fn get_hash(o: &mut Object, key: &str) -> Option<H256> {
	Some(H256::from(h32(o.remove(key)?.as_str()?)?))
}

/// mock ethheader
pub fn mock_header_from_source<'m>(o: &'m mut Object) -> Option<EthHeader> {
	let mixh = get_hash(o, "mixHash")?;
	let nonce = H64::from(h8(o.remove("nonce")?.as_str()?)?);
	let addr = h20(o.remove("miner")?.as_str()?)?;

	let extra_data = o.remove("extraData")?;
	let eds = &extra_data.as_str()?[2..];

	let log_bloom = o.remove("logsBloom")?;
	let lb = &log_bloom.as_str()?[2..];

	let difficulty: u64 = o.remove("difficulty")?.as_str()?.parse().unwrap();

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
