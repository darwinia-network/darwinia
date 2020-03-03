#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

pub use frame_support::traits::{LockIdentifier, WithdrawReason, WithdrawReasons};

pub use structs::*;
pub use traits::*;

pub mod structs {
	use codec::{Decode, Encode};
	use num_traits::Zero;

	use sp_runtime::{traits::AtLeast32Bit, RuntimeDebug};
	use sp_std::{cmp::Ordering, ops::BitOr, vec::Vec};

	use crate::{LockIdentifier, WithdrawReason, WithdrawReasons};

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
					acc + unbonding.amount
				} else {
					acc
				}
			})
		}

		#[inline]
		pub fn update_locks(&mut self, at: Moment) -> Balance {
			let mut locked_amount = self.staking_amount;

			self.unbondings.retain(|unbonding| {
				let valid = unbonding.valid_at(at);
				if valid {
					locked_amount += unbonding.amount;
				}

				valid
			});

			locked_amount
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
}

pub mod traits {
	use frame_support::traits::{Currency, ExistenceRequirement, TryDrop};
	use sp_runtime::DispatchResult;

	use crate::{LockFor, LockIdentifier, WithdrawReasons};

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
			withdraw_lock: LockFor<Self::Balance, Self::Moment>,
			reasons: WithdrawReasons,
		);

		/// Remove an existing lock.
		fn remove_lock(id: LockIdentifier, who: &AccountId);
	}

	// TODO doc
	pub trait Fee<AccountId, Balance> {
		fn pay_transfer_fee(
			transactor: &AccountId,
			transfer_fee: Balance,
			existence_requirement: ExistenceRequirement,
		) -> DispatchResult;
	}

	impl<AccountId, Balance> Fee<AccountId, Balance> for () {
		fn pay_transfer_fee(_: &AccountId, _: Balance, _: ExistenceRequirement) -> DispatchResult {
			Ok(())
		}
	}

	/// Callback on eth-backing module
	pub trait OnDepositRedeem<AccountId> {
		type Balance;
		type Moment;

		fn on_deposit_redeem(
			start_at: Self::Moment,
			months: Self::Moment,
			amount: Self::Balance,
			stash: &AccountId,
		) -> DispatchResult;
	}

	// FIXME: Ugly hack due to https://github.com/rust-lang/rust/issues/31844#issuecomment-557918823
	/// Handler for when some currency "account" decreased in balance for
	/// some reason.
	///
	/// The only reason at present for an increase would be for validator rewards, but
	/// there may be other reasons in the future or for other chains.
	///
	/// Reasons for decreases include:
	///
	/// - Someone got slashed.
	/// - Someone paid for a transaction to be included.
	pub trait OnUnbalancedKton<Imbalance: TryDrop> {
		/// Handler for some imbalance. Infallible.
		fn on_unbalanced(amount: Imbalance) {
			amount.try_drop().unwrap_or_else(Self::on_nonzero_unbalanced)
		}

		/// Actually handle a non-zero imbalance. You probably want to implement this rather than
		/// `on_unbalanced`.
		fn on_nonzero_unbalanced(amount: Imbalance);
	}

	impl<Imbalance: TryDrop> OnUnbalancedKton<Imbalance> for () {
		fn on_nonzero_unbalanced(amount: Imbalance) {
			drop(amount);
		}
	}
}
