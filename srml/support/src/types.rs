use codec::{Decode, Encode};
use rstd::vec::Vec;
use sr_primitives::{traits::SimpleArithmetic, RuntimeDebug};
use srml_support::traits::WithdrawReasons;

pub type TimeStamp = u64;

#[derive(Clone, Default, Encode, Decode, RuntimeDebug)]
pub struct BalanceLocks<Balance, Moment>(Vec<Lock<Balance, Moment>>);

impl<Balance, Moment> BalanceLocks<Balance, Moment>
where
	Balance: Clone + Copy + Default + SimpleArithmetic,
	Moment: Clone + Copy + PartialOrd,
{
	#[inline]
	fn update_lock(&mut self, lock: Lock<Balance, Moment>, at: Moment) -> Balance {
		let expired_locks_amount = self.remove_expired_locks(at);
		self.0.push(lock);

		expired_locks_amount
	}

	fn remove_locks(&mut self, at: Moment, lock: Lock<Balance, Moment>) -> Balance {
		let mut expired_locks_amount = Self::Balance::zero();

		<Locks<T>>::mutate(who, |locks| {
			locks.retain(|lock_| {
				if lock_.valid_at(now) && lock == lock {
					true
				} else {
					expired_locks_amount += lock.amount;
					false
				}
			});
		});

		expired_locks_amount
	}

	fn remove_expired_locks(&mut self, at: Moment) -> Balance {
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
}

#[derive(Clone, RuntimeDebug)]
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
}

#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct BalanceLock<Balance, Moment> {
	pub amount: Balance,
	pub at: Moment,
	pub reasons: WithdrawReasons,
}
