#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]
#![feature(drain_filter)]

use codec::{Decode, Encode};
use rstd::vec::Vec;
use sr_primitives::{
	traits::{SaturatedConversion, SimpleArithmetic},
	RuntimeDebug,
};
use srml_support::traits::Currency;
use srml_support::traits::WithdrawReasons;

//use super::traits::Locks as LocksTrait;

pub type TimeStamp = u64;

//#[derive(Clone, Default, PartialEq, Encode, Decode, RuntimeDebug)]
//pub struct Locks<Balance, Moment>(pub Vec<CompositeLock<Balance, Moment>>);
//
//impl<Balance, Moment> LocksTrait for Locks<Balance, Moment>
//where
//	Balance: Clone + Copy + Default + SimpleArithmetic,
//	Moment: Clone + Copy + PartialOrd,
//{
//	type Balance = Balance;
//	type Lock = CompositeLock<Balance, Moment>;
//	type Moment = Moment;
//	type WithdrawReasons = WithdrawReasons;
//
//	//	fn can_withdraw(&self, at: Self::Moment, reasons: Self::WithdrawReasons, new_balance: Self::Balance) -> bool {}
//}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub enum DetailLock<Balance, Moment> {
	BalanceDetailLock(BalanceLock<Balance, Moment>),
	StakingAndUnbondingDetailLock(StakingAndUnbondingLock<Balance, TimeStamp>),
}

impl<Balance, Moment> DetailLock<Balance, Moment>
where
	Balance: Clone + Copy + Default + SimpleArithmetic,
	Moment: Clone + Copy + PartialOrd + SaturatedConversion + rstd::convert::TryInto<u64>,
{
	pub fn valid_at(&self, at: Moment, new_balance: Balance) -> bool {
		match self {
			DetailLock::BalanceDetailLock(lock) => lock.valid_at(at, new_balance),
			DetailLock::StakingAndUnbondingDetailLock(lock) => {
				lock.valid_at(at.saturated_into::<TimeStamp>(), new_balance)
			}
		}
	}
}

impl<Balance, Moment> PartialEq for DetailLock<Balance, Moment>
where
	Balance: PartialEq,
	Moment: PartialEq,
{
	#[inline]
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(DetailLock::BalanceDetailLock(a), DetailLock::BalanceDetailLock(b)) => a == b,
			(DetailLock::StakingAndUnbondingDetailLock(a), DetailLock::StakingAndUnbondingDetailLock(b)) => a == b,
			_ => false,
		}
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct BalanceLock<Balance, Moment> {
	pub amount: Balance,
	pub until: Moment,
}

impl<Balance, Moment> BalanceLock<Balance, Moment>
where
	Balance: Clone + Copy + Default + SimpleArithmetic,
	Moment: Clone + Copy + PartialOrd,
{
	fn valid_at(&self, at: Moment, new_balance: Balance) -> bool {
		self.until > at && self.amount > new_balance
	}
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct UnlockChunk<Balance, Moment> {
	/// Amount of funds to be unlocked.
	// TODO: Compact?
	value: Balance,
	/// Era number at which point it'll be unlocked.
	until: Moment,
}

#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct StakingAndUnbondingLock<Balance, Moment> {
	pub staking_amount: Balance,
	// now >= l.until || new_balance >= l.amount
	/// Any balance that is becoming free, which may eventually be transferred out
	/// of the stash (assuming it doesn't get slashed first).
	pub unlocking: Vec<UnlockChunk<Balance, Moment>>,
}

impl<Balance, Moment> StakingAndUnbondingLock<Balance, Moment>
where
	Balance: Clone + Copy + Default + SimpleArithmetic,
	Moment: Clone + Copy + PartialOrd,
{
	//	// TODO:
	//	fn update_locks(&mut self, lock: UnlockChunk<Balance, Moment>, at: Moment) -> Balance {
	//		let expired_locks_amount = self.remove_expired_locks(at);
	//		// TODO add Eq to UnlockChunk
	//		if let Some(i) = self.unlocking.iter().position(|lock_| lock_ == &lock) {
	//			self.unlocking[i] = lock;
	//		} else {
	//			self.unlocking.push(lock);
	//		}
	//
	//		expired_locks_amount
	//	}
	//
	//	// TODO:
	//	fn remove_expired_locks(&mut self, at: Moment) -> Balance {
	//		let mut expired_locks_amount = Balance::default();
	//		self.unlocking.retain(|lock| {
	//			if lock.util > at {
	//				true
	//			} else {
	//				expired_locks_amount += lock.value;
	//				false
	//			}
	//		});
	//
	//		expired_locks_amount
	//	}
	//
	//	// TODO:
	//	fn remove_locks(&mut self, lock: &UnlockChunk<Balance, Moment>, at: Moment) -> Balance {
	//		let mut expired_locks_amount = Balance::default();
	//		self.unlocking.retain(|lock_| {
	//			if lock_.util > at && lock_ != lock {
	//				true
	//			} else {
	//				expired_locks_amount += lock_.value;
	//				false
	//			}
	//		});
	//
	//		expired_locks_amount
	//	}

	//
	//	/// The number of locks.
	//	fn locks_count(&self) -> u32;
	fn valid_at(&self, at: Moment, new_balance: Balance) -> bool {
		// TODO: Is it correct to use clone here?
		let mut locked_amount = self.staking_amount.clone();

		for lock in self.unlocking.iter() {
			if lock.until > at {
				// TODO: check overflow?
				locked_amount += lock.value;
			}
		}

		new_balance >= locked_amount
	}
}

//pub trait LockRate {
//    //TODO： ugly to use u64, ready for hacking
//    //    type Balance: SimpleArithmetic + As<usize> + As<u64> + Codec + Copy + MaybeSerializeDebug + Default;
//
//    fn bill_lock_rate() -> Perbill;
//
//    fn update_total_lock(amount: u64, is_add: bool) -> Result;
//}
//
//pub trait DarwiniaDilution<Balance> {
//    fn on_dilution(treasury_income: Balance);
//}

pub trait OnMinted<Balance> {
	fn on_minted(value: Balance);
}

pub trait OnAccountBalanceChanged<AccountId, Balance> {
	fn on_changed(who: &AccountId, old: Balance, new: Balance);
}

/// An identifier for a lock. Used for disambiguating different locks so that
/// they can be individually replaced or removed.
pub type LockIdentifier = [u8; 8];

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
		detail_lock: DetailLock<Self::Balance, Self::Moment>,
		reasons: WithdrawReasons,
	);

	/// Changes a balance lock (selected by `id`) so that it becomes less liquid in all
	/// parameters or creates a new one if it does not exist.
	///
	/// Calling `extend_lock` on an existing lock `id` differs from `set_lock` in that it
	/// applies the most severe constraints of the two, while `set_lock` replaces the lock
	/// with the new parameters. As in, `extend_lock` will set:
	/// - maximum `amount`
	/// - farthest duration (`until`)
	/// - bitwise mask of all `reasons`
	//	fn extend_lock(
	//		id: LockIdentifier,
	//		who: &AccountId,
	//		lock: DetailLock<Self::Balance, Self::Moment>,
	//		reasons: WithdrawReasons,
	//	);

	/// Remove an existing lock.
	fn remove_lock(id: LockIdentifier, who: &AccountId);
}
