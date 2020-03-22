mod check_receipt;
mod utils;

use json::JsonValue;

use crate::*;
use check_receipt::JSON;
use utils::*;

/// To help reward miners for when duplicate block solutions are found
/// because of the shorter block times of Ethereum (compared to other cryptocurrency).
/// An uncle is a smaller reward than a full block.
///
/// stackoverflow: https://ethereum.stackexchange.com/questions/34/what-is-an-uncle-ommer-block
///
/// returns: [grandpa, uncle, father, current]
pub fn mock_canonical_relationship() -> Option<Vec<Option<EthHeader>>> {
	if let JsonValue::Array(headers) = json::parse(JSON).unwrap().remove("headers") {
		Some(
			headers
				.iter()
				.map(|header| {
					if let JsonValue::Object(mut header) = header.clone() {
						mock_header_from_source(&mut header)
					} else {
						None
					}
				})
				.collect::<Vec<Option<EthHeader>>>(),
		)
	} else {
		None
	}
}
