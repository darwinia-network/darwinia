use codec::{Decode, Encode};
use sr_primitives::RuntimeDebug;
use srml_support::traits::WithdrawReasons;

pub type TimeStamp = u64;

#[derive(Clone, Eq, Encode, Decode, RuntimeDebug)]
pub struct BalanceLock<Balance, Moment> {
	pub amount: Balance,
	pub at: Moment,
	pub reasons: WithdrawReasons,
}

impl<Balance, Moment> Default for BalanceLock<Balance, Moment>
where
	Balance: Default,
	Moment: Default,
{
	fn default() -> Self {
		Self {
			amount: Balance::default(),
			at: Moment::default(),
			reasons: WithdrawReasons::all(),
		}
	}
}

impl<Balance, Moment> BalanceLock<Balance, Moment>
where
	Moment: PartialOrd,
{
	pub fn valid_at(&self, at: Moment) -> bool {
		self.at > at
	}
}

impl<Balance, Moment> PartialEq for BalanceLock<Balance, Moment>
where
	Moment: PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		self.at == other.at
	}
}
