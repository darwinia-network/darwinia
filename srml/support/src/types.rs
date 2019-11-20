use codec::{Decode, Encode};
use rstd::vec::Vec;
use sr_primitives::{traits::SimpleArithmetic, RuntimeDebug};
use srml_support::traits::WithdrawReasons;

use super::traits::Locks as LocksTrait;

pub type TimeStamp = u64;

#[derive(Clone, Default, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct Locks<Balance, Moment>(pub Vec<CompositeLock<Balance, Moment>>);

impl<Balance, Moment> LocksTrait for Locks<Balance, Moment>
where
	Balance: Clone + Copy + Default + SimpleArithmetic,
	Moment: Clone + Copy + PartialOrd,
{
	type Balance = Balance;
	type Lock = CompositeLock<Balance, Moment>;
	type Moment = Moment;
	type WithdrawReasons = WithdrawReasons;

	#[inline]
	fn locks_count(&self) -> u32 {
		self.0.len() as _
	}

	fn update_lock(&mut self, lock: Self::Lock, at: Self::Moment) -> Self::Balance {
		let expired_locks_amount = self.remove_expired_locks(at);
		if let Some(i) = self.0.iter().position(|lock_| lock_ == &lock) {
			self.0[i] = lock;
		} else {
			self.0.push(lock);
		}

		expired_locks_amount
	}

	fn remove_expired_locks(&mut self, at: Self::Moment) -> Self::Balance {
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

	fn remove_locks(&mut self, lock: &Self::Lock, at: Self::Moment) -> Self::Balance {
		let mut expired_locks_amount = Balance::zero();
		self.0.retain(|lock_| {
			if lock_.valid_at(at) && lock_ != lock {
				true
			} else {
				expired_locks_amount += lock_.amount();
				false
			}
		});

		expired_locks_amount
	}

	fn ensure_can_withdraw(
		&self,
		at: Self::Moment,
		reasons: Self::WithdrawReasons,
		new_balance: Self::Balance,
	) -> bool {
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

//impl<Balance, Moment> BalanceLocks<Balance, Moment>
//where
//	Balance: Clone + Copy + Default + SimpleArithmetic,
//	Moment: Clone + Copy + PartialOrd,
//{
//	#[inline]
//	pub fn len(&self) -> u32 {
//		self.0.len() as _
//	}
//
//	pub fn update_lock(&mut self, lock: Lock<Balance, Moment>, at: Moment) -> Balance {
//		let expired_locks_amount = self.remove_expired_locks(at);
//		if let Some(i) = self.0.iter().position(|lock_| lock_ == &lock) {
//			self.0[i] = lock;
//		} else {
//			self.0.push(lock);
//		}
//
//		expired_locks_amount
//	}
//
//	pub fn remove_expired_locks(&mut self, at: Moment) -> Balance {
//		let mut expired_locks_amount = Balance::default();
//		self.0.retain(|lock| {
//			if lock.valid_at(at) {
//				true
//			} else {
//				expired_locks_amount += lock.amount();
//				false
//			}
//		});
//
//		expired_locks_amount
//	}
//
//	pub fn remove_locks(&mut self, lock: &Lock<Balance, Moment>, at: Moment) -> Balance {
//		let mut expired_locks_amount = Balance::zero();
//		self.0.retain(|lock_| {
//			if lock_.valid_at(at) && lock_ != lock {
//				true
//			} else {
//				expired_locks_amount += lock_.amount();
//				false
//			}
//		});
//
//		expired_locks_amount
//	}
//
//	pub fn ensure_can_withdraw(&self, at: Moment, reasons: WithdrawReasons, new_balance: Balance) -> bool {
//		if self.0.is_empty() {
//			return true;
//		}
//
//		let mut locked_amount = Balance::default();
//		for lock in self.0.iter() {
//			if lock.valid_at(at) && lock.check_reasons_intersects(reasons) {
//				// TODO: check overflow?
//				locked_amount += lock.amount();
//			}
//		}
//
//		new_balance >= locked_amount
//	}
//}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub enum CompositeLock<Balance, Moment> {
	Staking(Balance),
	Unbonding(Lock<Balance, Moment>),
}

impl<Balance, Moment> CompositeLock<Balance, Moment>
where
	Balance: Copy,
	Moment: PartialOrd,
{
	#[inline]
	fn valid_at(&self, at: Moment) -> bool {
		match self {
			CompositeLock::Staking(_) => true,
			CompositeLock::Unbonding(balance_lock) => balance_lock.at > at,
		}
	}

	#[inline]
	fn amount(&self) -> Balance {
		match self {
			CompositeLock::Staking(balance) => *balance,
			CompositeLock::Unbonding(balance_lock) => balance_lock.amount,
		}
	}

	#[inline]
	fn check_reasons_intersects(&self, reasons: WithdrawReasons) -> bool {
		match self {
			CompositeLock::Staking(_) => true,
			CompositeLock::Unbonding(balance_lock) => balance_lock.reasons.intersects(reasons),
		}
	}
}

impl<Balance, Moment> PartialEq for CompositeLock<Balance, Moment>
where
	Moment: PartialEq,
{
	#[inline]
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(CompositeLock::Staking(_), CompositeLock::Staking(_)) => true,
			(CompositeLock::Unbonding(a), CompositeLock::Unbonding(b)) => a == b,
			_ => false,
		}
	}
}

#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct Lock<Balance, Moment> {
	pub amount: Balance,
	pub at: Moment,
	pub reasons: WithdrawReasons,
}
