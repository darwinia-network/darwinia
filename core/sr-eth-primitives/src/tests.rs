use crate::transaction::*;
use ethereum_types::{Address, H256, U128, U256};
use rustc_hex::FromHex;
use std::str::FromStr;
use support::{assert_err, assert_noop, assert_ok};

#[test]
/// kovan tx hash: 0xc654b4c4a183386722d42605ca91e23bc93919db8aa160b10cf50ab6a320ad9f
/// network: kovan
/// chain_id: 42
/// sender: 0x4cC4c344ebA849DC09ac9Af4bfF1977e44FC1D7E
/// gas_price: 15 Gwei
/// gas: 21000
/// action: eth transfer to 0x674943d6003783cf20125caad89525983dbfd050
/// sender nonce: 5240
fn test() {
	let bytes: Vec<u8> = FromHex::from_hex("f86e82147885037e11d60082520894674943d6003783cf20125caad89525983dbfd050881bc16d674ec800008078a01e4143882cd0b9b35710398205cd10e1aea773d938f3bfc10b278e6466bc79a0a05439639ccb7c41a79a7534bd7f3fb68a47b8c615b8a89c0c643fa3bcb7541e0a").unwrap();
	let tx: UnverifiedTransaction = rlp::decode(&bytes).expect("decoding failure");
	assert_eq!(tx.standard_v(), 1);
	assert_eq!(tx.original_v(), 0x78);
	// verify hash
	assert_eq!(
		tx.hash(),
		H256::from_str("c654b4c4a183386722d42605ca91e23bc93919db8aa160b10cf50ab6a320ad9f").unwrap()
	);
	// verify transaction fields
	assert_eq!(
		tx.as_unsigned(),
		&PlainTransaction {
			nonce: U256::from(U128::from(5240_u128)),
			gas_price: U256::from(U128::from(15000000000_u128)),
			gas: U256::from(U128::from(21000_u128)),
			action: Action::Call(Address::from_str("674943d6003783cf20125caad89525983dbfd050").unwrap()),
			value: U256::from(U128::from(2000000000000000000_u128)),
			data: b"".to_vec(),
		}
	);
}

#[test]
fn transaction_hash_should_be_derived_before() {
	let plain_tx = PlainTransaction {
		nonce: U256::from(U128::from(5240_u128)),
		gas_price: U256::from(U128::from(15000000000_u128)),
		gas: U256::from(U128::from(21000_u128)),
		action: Action::Call(Address::from_str("674943d6003783cf20125caad89525983dbfd050").unwrap()),
		value: U256::from(U128::from(2000000000000000000_u128)),
		data: b"".to_vec(),
	};

	// compute hash
	let r = H256::from_str("1e4143882cd0b9b35710398205cd10e1aea773d938f3bfc10b278e6466bc79a0").unwrap();
	println!("{:?}", r);
	let s = H256::from_str("5439639ccb7c41a79a7534bd7f3fb68a47b8c615b8a89c0c643fa3bcb7541e0a").unwrap();
	// standardV
	let v: u8 = 0x1;
	let signature = Signature::from_rsv(&r, &s, v);
	let unverified_tx = plain_tx.with_signature(signature, Some(42));
	assert_eq!(
		unverified_tx.hash(),
		H256::from_str("c654b4c4a183386722d42605ca91e23bc93919db8aa160b10cf50ab6a320ad9f").unwrap()
	);
}
