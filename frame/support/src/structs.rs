use codec::{Decode, Encode};
use num_traits::Zero;
use sp_runtime::{
	traits::{AtLeast32Bit, Saturating},
	RuntimeDebug,
};
use sp_std::{cmp::Ordering, ops::BitOr, prelude::*};

use crate::balance::lock::{LockIdentifier, WithdrawReason, WithdrawReasons};

/// Active balance information for an account.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct AccountData<Balance> {
	/// Non-reserved part of the balance. There may still be restrictions on this, but it is the
	/// total pool what may in principle be transferred, reserved and used for tipping.
	///
	/// This is the only balance that matters in terms of most operations on tokens. It
	/// alone is used to determine the balance when in the contract execution environment.
	pub free_ring: Balance,
	/// Non-reserved part of the balance. There may still be restrictions on this, but it is the
	/// total pool what may in principle be transferred, reserved and used for tipping.
	///
	/// This is the only balance that matters in terms of most operations on tokens. It
	/// alone is used to determine the balance when in the contract execution environment.
	pub free_kton: Balance,
	/// Balance which is reserved and may not be used at all.
	///
	/// This can still get slashed, but gets slashed last of all.
	///
	/// This balance is a 'reserve' balance that other subsystems use in order to set aside tokens
	/// that are still 'owned' by the account holder, but which are suspendable.
	pub reserved_ring: Balance,
	/// Balance which is reserved and may not be used at all.
	///
	/// This can still get slashed, but gets slashed last of all.
	///
	/// This balance is a 'reserve' balance that other subsystems use in order to set aside tokens
	/// that are still 'owned' by the account holder, but which are suspendable.
	pub reserved_kton: Balance,
}

impl<Balance> AccountData<Balance>
where
	Balance: Copy + Ord + Saturating + Zero,
{
	/// How much this account's balance can be reduced for the given `reasons`.
	pub fn usable_ring(&self, reasons: LockReasons, frozen_balance: FrozenBalance<Balance>) -> Balance {
		self.free_ring
			.saturating_sub(FrozenBalance::frozen_for(reasons, frozen_balance))
	}
	/// How much this account's balance can be reduced for the given `reasons`.
	pub fn usable_kton(&self, reasons: LockReasons, frozen_balance: FrozenBalance<Balance>) -> Balance {
		self.free_kton
			.saturating_sub(FrozenBalance::frozen_for(reasons, frozen_balance))
	}
	/// The total balance in this account including any that is reserved and ignoring any frozen.
	pub fn total_ring(&self) -> Balance {
		self.free_ring.saturating_add(self.reserved_ring)
	}
	/// The total balance in this account including any that is reserved and ignoring any frozen.
	pub fn total_kton(&self) -> Balance {
		self.free_kton.saturating_add(self.reserved_kton)
	}
}

/// Frozen balance information for an account.
pub struct FrozenBalance<Balance> {
	/// The amount that `free` may not drop below when withdrawing specifically for transaction
	/// fee payment.
	pub fee: Balance,
	/// The amount that `free` may not drop below when withdrawing for *anything except transaction
	/// fee payment*.
	pub misc: Balance,
}

impl<Balance> FrozenBalance<Balance>
where
	Balance: Copy + Ord + Zero,
{
	pub fn zero() -> Self {
		Self {
			fee: Zero::zero(),
			misc: Zero::zero(),
		}
	}

	/// The amount that this account's free balance may not be reduced beyond for the given
	/// `reasons`.
	pub fn frozen_for(reasons: LockReasons, frozen_balance: Self) -> Balance {
		match reasons {
			LockReasons::All => frozen_balance.misc.max(frozen_balance.fee),
			LockReasons::Misc => frozen_balance.misc,
			LockReasons::Fee => frozen_balance.fee,
		}
	}
}

/// Simplified reasons for withdrawing balance.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
pub enum LockReasons {
	/// Paying system transaction fees.
	Fee = 0,
	/// Any reason other than paying system transaction fees.
	Misc = 1,
	/// Any reason at all.
	All = 2,
}

impl From<WithdrawReasons> for LockReasons {
	fn from(r: WithdrawReasons) -> LockReasons {
		if r == WithdrawReasons::from(WithdrawReason::TransactionPayment) {
			LockReasons::Fee
		} else if r.contains(WithdrawReason::TransactionPayment) {
			LockReasons::All
		} else {
			LockReasons::Misc
		}
	}
}

impl BitOr for LockReasons {
	type Output = LockReasons;
	fn bitor(self, other: LockReasons) -> LockReasons {
		if self == other {
			return self;
		}
		LockReasons::All
	}
}

/// A single lock on a balance. There can be many of these on an account and they "overlap", so the
/// same balance is frozen by multiple locks.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct BalanceLock<Balance, Moment> {
	/// An identifier for this lock. Only one lock may be in existence for each identifier.
	pub id: LockIdentifier,
	pub lock_for: LockFor<Balance, Moment>,
	/// If true, then the lock remains in effect even for payment of transaction fees.
	pub lock_reasons: LockReasons,
}

#[cfg(feature = "easy-testing")]
impl<Balance, Moment> BalanceLock<Balance, Moment>
where
	Balance: Copy + PartialOrd + AtLeast32Bit,
	Moment: Copy + PartialOrd,
{
	// For performance, we don't need the `at` in some cases
	// Only use for tests to avoid write a lot of matches in tests
	pub fn locked_amount(&self, at: Option<Moment>) -> Balance {
		match &self.lock_for {
			LockFor::Common { amount } => *amount,
			LockFor::Staking(staking_lock) => {
				staking_lock.locked_amount(at.expect("This's a `StakingLock`, please specify the `Moment`."))
			}
		}
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub enum LockFor<Balance, Moment> {
	Common { amount: Balance },
	Staking(StakingLock<Balance, Moment>),
}

#[derive(Clone, Default, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct StakingLock<Balance, Moment> {
	/// The amount which the free balance may not drop below when this lock is in effect.
	pub staking_amount: Balance,
	pub unbondings: Vec<Unbonding<Balance, Moment>>,
}

impl<Balance, Moment> StakingLock<Balance, Moment>
where
	Balance: Copy + PartialOrd + AtLeast32Bit,
	Moment: Copy + PartialOrd,
{
	#[inline]
	pub fn locked_amount(&self, at: Moment) -> Balance {
		self.unbondings.iter().fold(self.staking_amount, |acc, unbonding| {
			if unbonding.valid_at(at) {
				acc.saturating_add(unbonding.amount)
			} else {
				acc
			}
		})
	}

	#[inline]
	pub fn update(&mut self, at: Moment) {
		let mut locked_amount = self.staking_amount;

		self.unbondings.retain(|unbonding| {
			let valid = unbonding.valid_at(at);
			if valid {
				locked_amount = locked_amount.saturating_add(unbonding.amount);
			}

			valid
		});
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct Unbonding<Balance, Moment> {
	/// The amount which the free balance may not drop below when this lock is in effect.
	pub amount: Balance,
	pub until: Moment,
}

impl<Balance, Moment> Unbonding<Balance, Moment>
where
	Balance: Copy + PartialOrd + Zero,
	Moment: PartialOrd,
{
	#[inline]
	fn valid_at(&self, at: Moment) -> bool {
		self.until > at
	}

	#[inline]
	pub fn locked_amount(&self, at: Moment) -> Balance {
		if self.valid_at(at) {
			self.amount
		} else {
			Zero::zero()
		}
	}
}

/// A wrapper for any rational number with a u32 bit numerator and denominator.
#[derive(Clone, Copy, Default, Eq, RuntimeDebug)]
pub struct Rational64(u64, u64);

impl Rational64 {
	/// Nothing.
	pub fn zero() -> Self {
		Self(0, 1)
	}

	/// If it is zero or not
	pub fn is_zero(&self) -> bool {
		self.0.is_zero()
	}

	/// Build from a raw `n/d`.
	pub fn from(n: u64, d: u64) -> Self {
		Self(n, d.max(1))
	}

	/// Build from a raw `n/d`. This could lead to / 0 if not properly handled.
	pub fn from_unchecked(n: u64, d: u64) -> Self {
		Self(n, d)
	}

	/// Return the numerator.
	pub fn n(&self) -> u64 {
		self.0
	}

	/// Return the denominator.
	pub fn d(&self) -> u64 {
		self.1
	}

	/// A saturating add that assumes `self` and `other` have the same denominator.
	pub fn lazy_saturating_add(self, other: Self) -> Self {
		if other.is_zero() {
			self
		} else {
			Self(self.0.saturating_add(other.0), self.1)
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
	pub fn multiply_by_rational(a: u64, b: u64, mut c: u64) -> u64 {
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

		((a as u128 * b as u128) / c as u128) as _
	}
}

impl PartialOrd for Rational64 {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Rational64 {
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
			let self_n = self.0 as u128 * other.1 as u128;
			let other_n = other.0 as u128 * self.1 as u128;
			self_n.cmp(&other_n)
		}
	}
}

impl PartialEq for Rational64 {
	fn eq(&self, other: &Self) -> bool {
		// handle some edge cases.
		if self.1 == other.1 {
			self.0.eq(&other.0)
		} else {
			let self_n = self.0 as u128 * other.1 as u128;
			let other_n = other.0 as u128 * self.1 as u128;
			self_n.eq(&other_n)
		}
	}
}
