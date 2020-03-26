mod check_receipt;
mod utils;

use crate::*;
use check_receipt::{EVENT_LOGS, JSON, RECEIPT};
use utils::*;

pub use utils::LogEntry;

/// To help reward miners for when duplicate block solutions are found
/// because of the shorter block times of Ethereum (compared to other cryptocurrency).
/// An uncle is a smaller reward than a full block.
///
/// stackoverflow: https://ethereum.stackexchange.com/questions/34/what-is-an-uncle-ommer-block
///
/// returns: [origin, grandpa, uncle, parent, current]
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

/// mock log events
pub fn mock_receipt_logs() -> Option<Vec<Option<LogEntry>>> {
	if let JsonValue::Array(logs) = json::parse(EVENT_LOGS).unwrap().remove("logs") {
		Some(
			logs.iter()
				.map(|log| {
					if let JsonValue::Object(mut log) = log.clone() {
						mock_log_from_source(&mut log)
					} else {
						None
					}
				})
				.collect::<Vec<Option<LogEntry>>>(),
		)
	} else {
		None
	}
}
