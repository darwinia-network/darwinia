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

		// mock header and proof
		let [_, header, _, _, _] = mock_canonical_relationship().unwrap();
		let proof_record = mock_canonical_receipt().unwrap();

		// mock logs
		let mut logs: Vec<LogEntry> = vec![];
		let mut log_entries = mock_receipt_logs().unwrap();
		for _ in 0..log_entries.len() {
			logs.push(log_entries.pop().unwrap().unwrap());
		}

		logs.reverse();

		// mock receipt
		let receipt = Receipt::new(TransactionOutcome::StatusCode(1), U256::from(U128::from(1371263)), logs);

		// verify receipt
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

		// relay grandpa
		assert_ok!(EthRelay::verify_header(&grandpa));
		assert_ok!(EthRelay::maybe_store_header(&grandpa));

		// relay parent
		assert_ok!(EthRelay::verify_header(&parent));
		assert_ok!(EthRelay::maybe_store_header(&parent));

		// relay current
		assert_ok!(EthRelay::verify_header(&current));
		assert_ok!(EthRelay::maybe_store_header(&current));
	});
}

#[test]
fn build_genesis_header() {
	let [genesis, _, _, _, _] = mock_canonical_relationship().unwrap();

	println!("{:?}", rlp::encode(&genesis.unwrap()));
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
