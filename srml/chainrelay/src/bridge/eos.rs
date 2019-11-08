use codec::{Decode, Encode};

use super::{Bridge, Relayer};
use crate::bridge::MerkleProof;

// TODO: type
type SucceededCount = ();
type FailedCount = ();
type EOSSpecField = ();

#[derive(Encode, Decode, Default)]
pub struct EOSBridge {}

impl Bridge for EOSBridge {
	fn push_header(&mut self, _: ()) -> Result<(), ()> {
		unimplemented!()
	}

	fn gen_merkle_proof(&self, _: ()) -> Result<MerkleProof, ()> {
		unimplemented!()
	}

	fn challenge(&self) -> Result<(), ()> {
		unimplemented!()
	}
}

#[derive(Encode, Decode, Default)]
pub struct EOSRelayer {
	succeeded: SucceededCount,
	failed: FailedCount,
	eos_spec_field: EOSSpecField,
}

impl Relayer for EOSRelayer {
	fn contribution(&self) -> () {
		unimplemented!()
	}

	fn failed(&self) -> () {
		unimplemented!()
	}

	fn reward(&mut self) -> () {
		unimplemented!()
	}

	fn punish(&mut self) -> () {
		unimplemented!()
	}
}
