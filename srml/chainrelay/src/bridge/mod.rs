pub mod eos;
pub mod ethereum;

pub use eos::*;
pub use ethereum::*;

// TODO: type
type Result<T> = core::result::Result<T, ()>;
type Header = ();
type Pos = ();
pub struct MerkleProof;
impl MerkleProof {
	pub fn verify(&self) -> Result<bool> {
		unimplemented!()
	}
}
type Contribution = ();
type Failed = ();
type Reward = ();
type Punish = ();

pub trait Bridge {
	fn push_header(&mut self, _: Header) -> Result<()>;
	fn gen_merkle_proof(&self, _: Pos) -> Result<MerkleProof>;
	fn verify_lock(&self, pos: Pos) -> Result<bool> {
		self.gen_merkle_proof(pos)?.verify()
	}
	// TODO: tacking PR: https://github.com/darwinia-network/rfcs/pull/29
	fn challenge(&self) -> Result<()>;
}

pub trait Relayer {
	fn contribution(&self) -> Contribution;
	fn failed(&self) -> Failed;
	fn reward(&mut self) -> Reward;
	fn punish(&mut self) -> Punish;
}
