mod check_receipt;
mod utils;

use json::JsonValue;

use crate::*;
use check_receipt::{JSON, RECEIPT};
use utils::*;

/// To help reward miners for when duplicate block solutions are found
/// because of the shorter block times of Ethereum (compared to other cryptocurrency).
/// An uncle is a smaller reward than a full block.
///
/// stackoverflow: https://ethereum.stackexchange.com/questions/34/what-is-an-uncle-ommer-block
///
/// returns: [grandpa, uncle, father, current]
pub fn mock_canonical_relationship() -> Option<[Option<EthHeader>; 5]> {
	if let JsonValue::Array(headers) = json::parse(JSON).unwrap().remove("headers") {
		let mut res: [Option<EthHeader>; 5] = [None, None, None, None, None];
		let mut hs = headers
			.iter()
			.map(|header| {
				if let JsonValue::Object(mut header) = header.clone() {
					mock_header_from_source(&mut header)
				} else {
					None
				}
			})
			.collect::<Vec<Option<EthHeader>>>();

		for i in 0..5 {
			std::mem::swap(&mut res[i], &mut hs[i]);
		}

		Some(res)
	} else {
		None
	}
}

/// mock canonical receipt
pub fn mock_canonical_receipt() -> Option<EthReceiptProof> {
	if let JsonValue::Object(mut receipt) = json::parse(RECEIPT).unwrap() {
		mock_receipt_from_source(&mut receipt)
	} else {
		None
	}
}
