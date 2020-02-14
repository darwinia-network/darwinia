#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

pub use frame_support::traits::{LockIdentifier, WithdrawReason, WithdrawReasons};

pub use structs::*;
pub use traits::*;

mod structs {
	use codec::{Decode, Encode};
	use num_traits::Zero;

	use sp_runtime::{traits::SimpleArithmetic, RuntimeDebug};
	use sp_std::{cmp::Ordering, vec::Vec};

	use crate::{LockIdentifier, WithdrawReasons};

	#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug)]
	pub struct BalanceLock<Balance, Moment> {
		pub id: LockIdentifier,
		pub withdraw_lock: WithdrawLock<Balance, Moment>,
		pub reasons: WithdrawReasons,
	}

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
	pub enum WithdrawLock<Balance, Moment> {
		Normal(NormalLock<Balance, Moment>),
		WithStaking(StakingLock<Balance, Moment>),
	}

	impl<Balance, Moment> WithdrawLock<Balance, Moment>
	where
		Balance: Copy + Default + SimpleArithmetic,
		Moment: Copy + PartialOrd,
	{
		pub fn can_withdraw(&self, at: Moment, new_balance: Balance) -> bool {
			match self {
				WithdrawLock::Normal(lock) => lock.can_withdraw(at, new_balance),
				WithdrawLock::WithStaking(lock) => lock.can_withdraw(at, new_balance),
			}
		}
	}

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
	pub struct NormalLock<Balance, Moment> {
		pub amount: Balance,
		pub until: Moment,
	}

	impl<Balance, Moment> NormalLock<Balance, Moment>
	where
		Balance: Copy + PartialOrd,
		Moment: PartialOrd,
	{
		#[inline]
		fn valid_at(&self, at: Moment) -> bool {
			self.until > at
		}

		#[inline]
		fn can_withdraw(&self, at: Moment, new_balance: Balance) -> bool {
			!self.valid_at(at) || self.amount <= new_balance
		}
	}

	#[derive(Clone, Default, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
	pub struct StakingLock<Balance, Moment> {
		pub staking_amount: Balance,
		pub unbondings: Vec<NormalLock<Balance, Moment>>,
	}

	impl<Balance, Moment> StakingLock<Balance, Moment>
	where
		Balance: Copy + PartialOrd + SimpleArithmetic,
		Moment: Copy + PartialOrd,
	{
		#[inline]
		fn can_withdraw(&self, at: Moment, new_balance: Balance) -> bool {
			let mut locked_amount = self.staking_amount;

			for unbonding in &self.unbondings {
				if unbonding.valid_at(at) {
					// TODO: check overflow?
					locked_amount += unbonding.amount;
				}
			}

			new_balance >= locked_amount
		}

		#[inline]
		pub fn shrink(&mut self, at: Moment) {
			self.unbondings.retain(|unbonding| unbonding.valid_at(at));
		}
	}

	/// A wrapper for any rational number with a u32 bit numerator and denominator.
	#[derive(Clone, Copy, Default, Eq, RuntimeDebug)]
	pub struct Rational32(u32, u32);

	impl Rational32 {
		/// Nothing.
		pub fn zero() -> Self {
			Self(0, 1)
		}

		/// If it is zero or not
		pub fn is_zero(&self) -> bool {
			self.0.is_zero()
		}

		/// Build from a raw `n/d`.
		pub fn from(n: u32, d: u32) -> Self {
			Self(n, d.max(1))
		}

		/// Build from a raw `n/d`. This could lead to / 0 if not properly handled.
		pub fn from_unchecked(n: u32, d: u32) -> Self {
			Self(n, d)
		}

		/// Return the numerator.
		pub fn n(&self) -> u32 {
			self.0
		}

		/// Return the denominator.
		pub fn d(&self) -> u32 {
			self.1
		}

		/// A saturating add that assumes `self` and `other` have the same denominator.
		pub fn lazy_add(self, other: Self) -> Self {
			if other.is_zero() {
				self
			} else {
				Self(self.0 + other.0, self.1)
			}
		}

		/// A saturating subtraction that assumes `self` and `other` have the same denominator.
		pub fn lazy_saturating_sub(self, other: Self) -> Self {
			if other.is_zero() {
				self
			} else {
				Self(self.0.saturating_sub(other.0), self.1)
			}
		}

		/// Safely and accurately compute `a * b / c`. The approach is:
		///   - Simply try `a * b / c`.
		///   - Else, convert them both into big numbers and re-try.
		///
		/// Invariant: c must be greater than or equal to 1.
		pub fn multiply_by_rational(a: u32, b: u32, mut c: u32) -> u32 {
			if a.is_zero() || b.is_zero() {
				return 0;
			}
			c = c.max(1);

			// a and b are interchangeable by definition in this function. It always helps to assume the
			// bigger of which is being multiplied by a `0 < b/c < 1`. Hence, a should be the bigger and
			// b the smaller one.
			let (mut a, mut b) = if a > b { (a, b) } else { (b, a) };

			// Attempt to perform the division first
			if a % c == 0 {
				a /= c;
				c = 1;
			} else if b % c == 0 {
				b /= c;
				c = 1;
			}

			((a as u64 * b as u64) / c as u64) as _
		}
	}

	impl PartialOrd for Rational32 {
		fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
			Some(self.cmp(other))
		}
	}

	impl Ord for Rational32 {
		fn cmp(&self, other: &Self) -> Ordering {
			// handle some edge cases.
			if self.1 == other.1 {
				self.0.cmp(&other.0)
			} else if self.1.is_zero() {
				Ordering::Greater
			} else if other.1.is_zero() {
				Ordering::Less
			} else {
				// Don't even compute gcd.
				let self_n = self.0 as u64 * other.1 as u64;
				let other_n = other.0 as u64 * self.1 as u64;
				self_n.cmp(&other_n)
			}
		}
	}

	impl PartialEq for Rational32 {
		fn eq(&self, other: &Self) -> bool {
			// handle some edge cases.
			if self.1 == other.1 {
				self.0.eq(&other.0)
			} else {
				let self_n = self.0 as u64 * other.1 as u64;
				let other_n = other.0 as u64 * self.1 as u64;
				self_n.eq(&other_n)
			}
		}
	}
}

mod traits {
	use frame_support::traits::{Currency, ExistenceRequirement};
	use sp_runtime::DispatchResult;

	use crate::{LockIdentifier, WithdrawLock, WithdrawReasons};

	/// A currency whose accounts can have liquidity restrictions.
	pub trait LockableCurrency<AccountId>: Currency<AccountId> {
		/// The quantity used to denote time; usually just a `BlockNumber`.
		type Moment;

		/// Create a new balance lock on account `who`.
		///
		/// If the new lock is valid (i.e. not already expired), it will push the struct to
		/// the `Locks` vec in storage. Note that you can lock more funds than a user has.
		///
		/// If the lock `id` already exists, this will update it.
		fn set_lock(
			id: LockIdentifier,
			who: &AccountId,
			withdraw_lock: WithdrawLock<Self::Balance, Self::Moment>,
			reasons: WithdrawReasons,
		);

		/// Remove an existing lock.
		fn remove_lock(id: LockIdentifier, who: &AccountId);
	}

	pub trait Fee<AccountId, Balance> {
		fn pay_transfer_fee(
			transactor: &AccountId,
			transfer_fee: Balance,
			existence_requirement: ExistenceRequirement,
		) -> DispatchResult;
	}

	// callback on eth-backing module
	pub trait OnDepositRedeem<AccountId> {
		type Moment;

		fn on_deposit_redeem(months: u64, start_at: u64, amount: u128, stash: &AccountId) -> Result<(), &'static str>;
	}
}
