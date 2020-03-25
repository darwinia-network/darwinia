//! Tests for eth-relay.

use eth_primitives::{receipt::TransactionOutcome, U128};
use frame_support::{assert_err, assert_ok};
use frame_system::RawOrigin;

use crate::{mock::*, mock_headers::*, *};

#[test]
fn verify_receipt_proof() {
	new_test_ext().execute_with(|| {
		System::inc_account_nonce(&2);
		assert_ok!(EthRelay::set_number_of_blocks_safe(RawOrigin::Root.into(), 0));

		let [_, header, _, _, _] = mock_canonical_relationship().unwrap();
		let proof_record = mock_canonical_receipt().unwrap();

		let mut logs: Vec<LogEntry> = vec![];
		let mut log_entries = mock_receipt_logs().unwrap();
		for _ in 0..log_entries.len() {
			logs.push(log_entries.pop().unwrap().unwrap());
		}
		logs.reverse();

		let receipt = Receipt::new(TransactionOutcome::StatusCode(1), U256::from(U128::from(1371263)), logs);

		assert_ok!(EthRelay::init_genesis_header(&header.unwrap(), 0x624c22d93f8e59_u64));
		assert_eq!(EthRelay::verify_receipt(&proof_record), Ok(receipt));
	});
}

#[test]
fn relay_header() {
	new_test_ext().execute_with(|| {
		let [o, g, _, p, c] = mock_canonical_relationship().unwrap();
		let [origin, grandpa, parent, current] = [o.unwrap(), g.unwrap(), p.unwrap(), c.unwrap()];
		assert_ok!(EthRelay::init_genesis_header(&origin, 0x624c22d93f8e59_u64));

		assert_ok!(EthRelay::verify_header(&grandpa));
		assert_ok!(EthRelay::maybe_store_header(&grandpa));

		assert_ok!(EthRelay::verify_header(&parent));
		assert_ok!(EthRelay::maybe_store_header(&parent));

		assert_ok!(EthRelay::verify_header(&current));
		assert_ok!(EthRelay::maybe_store_header(&current));
	});
}

#[test]
fn build_genesis_header() {
	let genesis_header = EthHeader {
			parent_hash: hex!("6e1494ad8e02a126fb86cf82b13627c45e6e8f1eab05e14270a69a05d7fe5e26").into(),
			timestamp: 0x5e78a9f5,
			number: 0x946eef,
			author: hex!("5a0b54d5dc17e0aadc383d2db43b0a0d3e029c4c").into(),
			transactions_root: hex!("661aff2eca6c57b3cb0f0b55d44761d4269c6548ed12bdebaa37df36d2e5b758").into(),
			uncles_hash: hex!("8652a5fec23f8bd33f9898c05ccd953c2c1faeb0d3de918c967a24b709254814").into(),
			extra_data: "737061726b706f6f6c2d6574682d636e2d687a33".from_hex().unwrap(),
			state_root: hex!("53fceb8f891e321c5124414a9bfd97ee39abb975066936bd220597a578c7655a").into(),
			receipts_root: hex!("4b57744cd97d237a8751a96317aecbe7db52f302ded36246d41782face81c17c").into(),
			log_bloom: Bloom::from_str("7890cc80915ca44051c6e0c101505084edc980e151012010b46c00623e4020a83a581359d08b095ead0116a408da0b9c3782605088210826133440ea6824981c78250060f64aa3a6c890102800c235e7204164252648a4e5a240e6e72068000030320104045c412de0ae448a126c10400e244864500b2c249e00aeb061143064b7b810d4e601a018542542c095880c521b89853b45840018616b0816ce90f2c01a642124b20c3d008cfbe08702607ba4f268200c294e1c2002b3280f3aae9312119421e2570840bb40233131064b408d3a003378994005c1090a8073c1501493b053ecc480ca50185e8105d240762a670ca43a6036408ab46204d21e0c923d1a").unwrap(),
			gas_used: 0x983707.into(),
			gas_limit: 0x9883be.into(),
			difficulty: 0x7db16f1a4402eu64.into(),
			seal: vec![
				rlp::encode(&H256::from(hex!("1cf81d78588bedf4ef8a0db007bba31b17c1086bead9b9badeca8d34b15db420"))), 
				rlp::encode(&H64::from(hex!("a98d18400422c64e"))),
			],
			hash: Some(hex!("034cd83d9150808de742592f57e302dc9eccc71af270639dc5f236e5bdd7d3e3").into()),
	};

	println!("{:?}", rlp::encode(&genesis_header));
}

/// # Check Receipt Safety
///
/// ## Family Tree
///
/// | pos     | height  | tx                                                                 |
/// |---------|---------|--------------------------------------------------------------------|
/// | origin  | 7575765 |                                                                    |
/// | grandpa | 7575766 | 0xc56be493f656f1c8222006eda5cd3392be5f0c096e8b7fb1c5542088c0f0c889 |
/// | uncle   | 7575766 |                                                                    |
/// | parent  | 7575767 |                                                                    |
/// | current | 7575768 | 0xfc836bf547f1e035e837bf0a8d26e432aa26da9659db5bf6ba69b0341d818778 |
///
/// To help reward miners for when duplicate block solutions are found
/// because of the shorter block times of Ethereum (compared to other cryptocurrency).
/// An uncle is a smaller reward than a full block.
///
/// ## Note:
///
/// check receipt should
/// - succeed when we relayed the correct header
/// - failed when canonical hash was re-orged by the block which contains our tx's brother block
#[test]
fn check_receipt_safety() {
	new_test_ext().execute_with(|| {
		assert_ok!(EthRelay::add_authority(RawOrigin::Root.into(), 0));
		assert_ok!(EthRelay::set_number_of_blocks_safe(RawOrigin::Root.into(), 0));

		// family tree
		let [o, g, u, _p, _c] = mock_canonical_relationship().unwrap();
		let [origin, grandpa, uncle] = [o.unwrap(), g.unwrap(), u.unwrap()];
		assert_ok!(EthRelay::init_genesis_header(&origin, 0x624c22d93f8e59_u64));

		let receipt = mock_canonical_receipt().unwrap();
		assert_ne!(grandpa.hash, uncle.hash);
		assert_eq!(grandpa.number, uncle.number);

		// check receipt should succeed when we relayed the correct header
		assert_ok!(EthRelay::relay_header(Origin::signed(0), grandpa.clone()));
		assert_ok!(EthRelay::check_receipt(Origin::signed(0), receipt.clone()));

		// check should fail when canonical hash was re-orged by
		// the block which contains our tx's brother block
		assert_ok!(EthRelay::relay_header(Origin::signed(0), uncle));
		assert_err!(
			EthRelay::check_receipt(Origin::signed(0), receipt.clone()),
			<Error<Test>>::HeaderNC
		);
	});
}

#[test]
fn canonical_reorg_uncle_should_succeed() {
	new_test_ext().execute_with(|| {
		assert_ok!(EthRelay::add_authority(RawOrigin::Root.into(), 0));
		assert_ok!(EthRelay::set_number_of_blocks_safe(RawOrigin::Root.into(), 0));

		let [o, g, u, _p, _c] = mock_canonical_relationship().unwrap();
		let [origin, grandpa, uncle] = [o.unwrap(), g.unwrap(), u.unwrap()];
		assert_ok!(EthRelay::init_genesis_header(&origin, 0x624c22d93f8e59_u64));

		// check relationship
		assert_ne!(grandpa.hash, uncle.hash);
		assert_eq!(grandpa.number, uncle.number);

		let (gh, uh) = (grandpa.hash, uncle.hash);
		let number = grandpa.number;

		// relay uncle header
		assert_ok!(EthRelay::relay_header(Origin::signed(0), uncle));
		assert_eq!(EthRelay::canonical_header_hash_of(number), uh.unwrap());

		// relay grandpa and re-org uncle
		assert_ok!(EthRelay::relay_header(Origin::signed(0), grandpa));
		assert_eq!(EthRelay::canonical_header_hash_of(number), gh.unwrap());
	});
}

#[test]
fn test_safety_block() {
	new_test_ext().execute_with(|| {
		assert_ok!(EthRelay::add_authority(RawOrigin::Root.into(), 0));
		assert_ok!(EthRelay::set_number_of_blocks_safe(RawOrigin::Root.into(), 2));

		// family tree
		let [o, g, p, u, c] = mock_canonical_relationship().unwrap();
		let [origin, grandpa, parent, uncle, current] = [o.unwrap(), g.unwrap(), p.unwrap(), u.unwrap(), c.unwrap()];

		let receipt = mock_canonical_receipt().unwrap();

		// not safety after 0 block
		assert_ok!(EthRelay::init_genesis_header(&origin, 0x624c22d93f8e59_u64));
		assert_ok!(EthRelay::relay_header(Origin::signed(0), grandpa));
		assert_err!(
			EthRelay::check_receipt(Origin::signed(0), receipt.clone()),
			<Error<Test>>::HeaderNS
		);

		// not safety after 2 blocks
		assert_ok!(EthRelay::relay_header(Origin::signed(0), parent));
		assert_ok!(EthRelay::relay_header(Origin::signed(0), uncle));
		assert_err!(
			EthRelay::check_receipt(Origin::signed(0), receipt.clone()),
			<Error<Test>>::HeaderNS
		);

		// safety after 3 blocks
		assert_ok!(EthRelay::relay_header(Origin::signed(0), current));
		assert_ok!(EthRelay::check_receipt(Origin::signed(0), receipt));
	});
}
