//! Tests for the module.

#![cfg(test)]

use super::*;
use crate::mock::*;
//use codec::{Decode, Encode};
use frame_system::{EventRecord, Phase};
use sp_runtime::{
	testing::{Digest, H256},
	traits::{Header, OnFinalize},
};

fn initialize_block(number: u64, parent_hash: H256) {
	System::initialize(
		&number,
		&parent_hash,
		&Default::default(),
		&Default::default(),
		Default::default(),
	);
}

#[test]
fn first_header_mmr() {
	new_test_ext().execute_with(|| {
		let parent_hash: H256 = Default::default();
		initialize_block(1, parent_hash);

		System::note_finished_extrinsics();
		HeaderMMR::on_finalize(1);

		let header = System::finalize();
		assert_eq!(
			header.digest,
			Digest {
				logs: vec![header_mmr_log(parent_hash)]
			}
		);

		assert_eq!(
			System::events(),
			vec![EventRecord {
				phase: Phase::Finalization,
				event: Event::<Test>::NewMMRRoot(parent_hash).into(),
				topics: vec![],
			},]
		);
	});
}

#[test]
fn test_insert_header() {
	new_test_ext().execute_with(|| {
		initialize_block(1, Default::default());

		HeaderMMR::on_finalize(1);

		let mut headers = vec![];

		let mut header = System::finalize();
		headers.push(header.clone());

		for i in 2..30 {
			initialize_block(i, header.hash());

			HeaderMMR::on_finalize(i);
			header = System::finalize();
			headers.push(header.clone());
		}

		let h1 = 11 as u64;
		let h2 = 19 as u64;

		let prove_elem = headers[h1 as usize - 1].hash();

		let pos = 19;
		assert_eq!(pos, HeaderMMR::position_of(h1));
		assert_eq!(prove_elem, HeaderMMR::mmr_node_list(pos));

		let mmr_root = headers[h2 as usize - 1]
			.digest
			.convert_first(|l| l.as_merkle_mountain_range_root().cloned())
			.expect("Header mmr get failed");

		let store = ModuleMMRStore::<Test>::default();
		let mmr = MMR::<_, MMRMerge<Test>, _>::new(HeaderMMR::position_of(h2), store);

		assert_eq!(mmr.get_root().expect("Get Root Failed"), mmr_root);

		let proof = HeaderMMR::_gen_proof(h1, h2).expect("gen proof");

		let result = proof.verify(mmr_root, vec![(pos, prove_elem)]).expect("verify");
		assert!(result);
	});
}
