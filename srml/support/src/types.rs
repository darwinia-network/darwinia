use codec::{Decode, Encode};
use rstd::vec::Vec;
use sr_primitives::{traits::Zero, RuntimeDebug};
use srml_support::traits::WithdrawReasons;

pub type TimeStamp = u64;

#[derive(Clone, PartialEq, Default, Encode, Decode, RuntimeDebug)]
pub struct CompositeLock<Balance, Moment> {
	pub staking_amount: Balance,
	pub locks: Vec<BalanceLock<Balance, Moment>>,
}

impl<Balance, Moment> CompositeLock<Balance, Moment>
where
	Balance: Zero,
{
	pub fn is_empty(&self) -> bool {
		self.staking_amount.is_zero() && self.locks.is_empty()
	}
}

pub struct LockUpdateStrategy<Balance, Moment> {
	/// if `lock` is set, `check_expired` will be ignored
	pub check_expired: bool,
	pub staking_amount: Option<Balance>,
	pub lock: Option<BalanceLock<Balance, Moment>>,
}

impl<Balance, Moment> LockUpdateStrategy<Balance, Moment> {
	pub fn new() -> Self {
		Self {
			check_expired: false,
			staking_amount: None,
			lock: None,
		}
	}

	pub fn with_check_expired(mut self, check_expired: bool) -> Self {
		self.check_expired = check_expired;
		self
	}

	pub fn with_staking_amount(mut self, staking_amount: Balance) -> Self {
		self.staking_amount = Some(staking_amount);
		self
	}

	pub fn with_lock(mut self, lock: BalanceLock<Balance, Moment>) -> Self {
		self.lock = Some(lock);
		self
	}
}

#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct BalanceLock<Balance, Moment> {
	pub amount: Balance,
	pub at: Moment,
	pub reasons: WithdrawReasons,
}

impl<Balance, Moment> BalanceLock<Balance, Moment>
where
	Moment: PartialOrd,
{
	pub fn valid_at(&self, at: Moment) -> bool {
		self.at > at
	}
}
