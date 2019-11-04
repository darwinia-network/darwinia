//use sr_primitives::Hash;
use substrate_primitives::U256;

type Bytes = Vec<u8>;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
	/// It's a call action.
	Call(Call),
	/// It's a create action.
	Create(Create),
	/// Suicide.
	Suicide(Suicide),
	/// Reward
	Reward(Reward),
}

pub struct BestBLock<Hash> {
	height: u64, // enough for ethereum poa network (kovan)
	hash: Hash,
	total_difficulty: U256,
}

struct PoaHeader<Hash, Address, Moment> {
	parent_hash: Hash,
	ommers_hash: Hash,
	beneficiary: Address,
	state_root: Hash,
	transactions_root: Hash,
	receipt_root: Hash,
	logs_bloom: Hash,
	difficulty: u64,
	number: u64,
	gas_limit: u64,
	gas_used: u64,
	timestamp: Moment,
	extra_data: Bytes,
	mix_hash: Hash,
	nonce: u64,
}

pub struct Transaction<Hash> {
	pub nonce: U256,
	pub gas_price: U256,
	pub gas: U256,
	pub action: Action,
	pub value: U256,
	pub data: Bytes,
}

#[derive(Debug, Clone, Eq, PartialEq, MallocSizeOf)]
pub struct UnverifiedTransaction<Hash> {
	/// Plain Transaction.
	unsigned: Transaction<Hash>,
	/// The V field of the signature; the LS bit described which half of the curve our point falls
	/// in. The MS bits describe which chain this transaction is for. If 27/28, its for all chains.
	v: u64,
	/// The R field of the signature; helps describe the point on the curve.
	r: U256,
	/// The S field of the signature; helps describe the point on the curve.
	s: U256,
	/// Hash of the transaction
	hash: Hash,
}
