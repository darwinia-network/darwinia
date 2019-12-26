#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

pub use srml_support::traits::{LockIdentifier, WithdrawReason, WithdrawReasons};

pub use structs::*;
pub use traits::*;

mod structs {
	use codec::{Decode, Encode};
	use rstd::vec::Vec;
	use sr_primitives::{traits::SimpleArithmetic, RuntimeDebug};

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
}

mod traits {
	use rstd::result;
	use srml_support::traits::Currency;

	use crate::{LockIdentifier, WithdrawLock, WithdrawReasons};

	pub trait OnMinted<Balance> {
		fn on_minted(value: Balance);
	}

	pub trait OnAccountBalanceChanged<AccountId, Balance> {
		fn on_changed(who: &AccountId, old: Balance, new: Balance);
	}

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

	// callback on eth-backing module
	pub trait OnDepositRedeem<AccountId> {
		type Moment;

		fn on_deposit_redeem(
			months: u64,
			start_at: u64,
			amount: u128,
			stash: &AccountId,
		) -> result::Result<(), &'static str>;
	}
}
