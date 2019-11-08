use codec::{Decode, Encode};

use super::{Bridge, Relayer};
use crate::bridge::MerkleProof;

// TODO: type
type SucceededCount = ();
type FailedCount = ();
type EthereumSpecField = ();

#[derive(Encode, Decode, Default)]
pub struct EthereumBridge {}

impl Bridge for EthereumBridge {
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
pub struct EthereumRelayer {
	succeeded: SucceededCount,
	failed: FailedCount,
	ethereum_spec_field: EthereumSpecField,
}

impl Relayer for EthereumRelayer {
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
