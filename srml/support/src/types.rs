use codec::{Decode, Encode};
use sr_primitives::RuntimeDebug;
use srml_support::traits::WithdrawReasons;

pub type TimeStamp = u64;

#[derive(Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct BalanceLock<Balance, Moment> {
	pub id: Id<Moment>,
	pub amount: Balance,
	pub reasons: WithdrawReasons,
}

impl<Balance, Moment> BalanceLock<Balance, Moment>
where
	Moment: PartialOrd,
{
	pub fn valid_at(&self, at: &Moment) -> bool {
		self.id.until() > at
	}
}

impl<Balance, Moment> PartialEq for BalanceLock<Balance, Moment>
where
	Moment: PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

#[derive(Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum Id<Moment> {
	Staking(Moment),
	Unbonding(Moment),
}

impl<Moment> Id<Moment> {
	fn until(&self) -> &Moment {
		match self {
			Id::Staking(moment) => moment,
			Id::Unbonding(moment) => moment,
		}
	}
}

impl<Moment> PartialEq for Id<Moment>
where
	Moment: PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		match self {
			Id::Staking(_) => match other {
				Id::Staking(_) => true,
				_ => false,
			},
			Id::Unbonding(moment) => match other {
				Id::Unbonding(moment_) => moment == moment_,
				_ => false,
			},
		}
	}
}
