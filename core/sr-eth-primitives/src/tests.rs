use super::*;
use ethereum_types::{Address, H256, U128, U256};
use rustc_hex::FromHex;
use std::str::FromStr;
use support::{assert_err, assert_noop, assert_ok};

#[test]
fn test() {
	let bytes: Vec<u8> = FromHex::from_hex("f85f800182520894095e7baea6a6c7c4c2dfeb977efac326af552d870a801ba048b55bfa915ac795c431978d8a6a992b628d557da5ff759b307d495a36649353a0efffd310ac743f371de3b9f7f9cb56c0b28ad43601b4ab949f53faa07bd2c804").unwrap();
	let tx: UnverifiedTransaction = rlp::decode(&bytes).expect("decoding failure");

	assert_eq!(tx.unsigned.gas, U256::from(U128::from(21000)));
}

#[test]
fn compute_transaction_hash() {
	let plain_tx = PlainTransaction {
		nonce: U256::from(U128::from(5240 as u128)),
		gas_price: U256::from(U128::from(15000000000 as u128)),
		gas: U256::from(U128::from(21000 as u128)),
		action: Action::Call(Address::from_str("674943d6003783cf20125caad89525983dbfd050").unwrap()),
		value: U256::from(U128::from(2000000000000000000 as u128)),
		data: b"".to_vec(),
	};

	// fill it with secret key
	let sec = Secret::from_str("xxxxx").unwrap();
	let signed_tx = plain_tx.sign(&sec, Some(42));
	assert_eq!(
		&signed_tx.hash(),
		&H256::from_str("c654b4c4a183386722d42605ca91e23bc93919db8aa160b10cf50ab6a320ad9f").unwrap()
	);
}
