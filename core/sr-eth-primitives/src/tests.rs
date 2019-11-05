use super::*;
use ethereum_types::{Address, U128, U256};
use rustc_hex::FromHex;
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
		nonce:
	}
}
