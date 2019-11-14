use codec::{Decode, Encode};
use sr_primitives::RuntimeDebug;

pub type TimeStamp = u64;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum Id<Moment> {
	Staking,
	Unbonding(Moment),
}
