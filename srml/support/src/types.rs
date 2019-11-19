use codec::{Decode, Encode};
use rstd::vec::Vec;
use sr_primitives::{traits::SimpleArithmetic, RuntimeDebug};
use srml_support::traits::WithdrawReasons;

pub type TimeStamp = u64;

#[derive(Clone, Default, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct BalanceLocks<Balance, Moment>(pub Vec<Lock<Balance, Moment>>);

impl<Balance, Moment> BalanceLocks<Balance, Moment>
where
	Balance: Clone + Copy + Default + SimpleArithmetic,
	Moment: Clone + Copy + PartialOrd,
{
	#[inline]
	pub fn len(&self) -> u32 {
		self.0.len() as _
	}

	#[inline]
	pub fn update_lock(&mut self, lock: Lock<Balance, Moment>, at: Moment) -> Balance {
		let expired_locks_amount = self.remove_locks(at, &lock);
		self.0.push(lock);

		expired_locks_amount
	}

	pub fn remove_locks(&mut self, at: Moment, lock: &Lock<Balance, Moment>) -> Balance {
		let mut expired_locks_amount = Balance::zero();
		self.0.retain(|lock_| {
			if lock_.valid_at(at) && lock_ == lock {
				true
			} else {
				expired_locks_amount += lock.amount();
				false
			}
		});

		expired_locks_amount
	}

	pub fn remove_expired_locks(&mut self, at: Moment) -> Balance {
		let mut expired_locks_amount = Balance::default();
		self.0.retain(|lock| {
			if lock.valid_at(at) {
				true
			} else {
				expired_locks_amount += lock.amount();
				false
			}
		});

		expired_locks_amount
	}

	pub fn can_withdraw(&self, at: Moment, reasons: WithdrawReasons, new_balance: Balance) -> bool {
		if self.0.is_empty() {
			return true;
		}

		let mut locked_amount = Balance::default();
		for lock in self.0.iter() {
			if lock.valid_at(at) && lock.check_reasons_intersects(reasons) {
				// TODO: check overflow?
				locked_amount += lock.amount();
			}
		}

		new_balance >= locked_amount
	}
}

#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum Lock<Balance, Moment> {
	Staking(Balance),
	Unbonding(BalanceLock<Balance, Moment>),
}

impl<Balance, Moment> Lock<Balance, Moment>
where
	Balance: Copy,
	Moment: PartialOrd,
{
	#[inline]
	fn valid_at(&self, at: Moment) -> bool {
		match self {
			Lock::Staking(_) => true,
			Lock::Unbonding(balance_lock) => balance_lock.at > at,
		}
	}

	#[inline]
	fn amount(&self) -> Balance {
		match self {
			Lock::Staking(balance) => *balance,
			Lock::Unbonding(balance_lock) => balance_lock.amount,
		}
	}

	#[inline]
	fn check_reasons_intersects(&self, reasons: WithdrawReasons) -> bool {
		match self {
			Lock::Staking(_) => true,
			Lock::Unbonding(balance_lock) => balance_lock.reasons.intersects(reasons),
		}
	}
}

#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct BalanceLock<Balance, Moment> {
	pub amount: Balance,
	pub at: Moment,
	pub reasons: WithdrawReasons,
}
