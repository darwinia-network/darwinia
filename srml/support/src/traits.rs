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

use srml_support::traits::Currency;

pub trait OnMinted<Balance> {
	fn on_minted(value: Balance);
}

pub trait OnAccountBalanceChanged<AccountId, Balance> {
	fn on_changed(who: &AccountId, old: Balance, new: Balance);
}

/// A more powerful lockable currency.
pub trait LockableCurrency<AccountId>: Currency<AccountId> {
	type Lock;
	/// The quantity used to denote time; usually just a `BlockNumber`.
	/// In Darwinia we prefer using `TimeStamp/u64`.
	type Moment;
	type WithdrawReasons;

	/// The number of locks.
	fn locks_count(who: &AccountId) -> u32;

	/// - Create a new balance lock on account `who`.
	/// 	- If the new lock is valid (i.e. not already expired), it will push the struct to
	/// 	the `Locks` vec in storage. Note that you can lock more funds than a user has.
	/// 	- If the lock `id/until` already exists, this will update it.
	/// - Remove the expired locks on account `who`.
	/// - Update the global staking amount.
	/// - The function will return the sum of expired locks' amount.
	fn update_lock(who: &AccountId, lock: Option<Self::Lock>) -> Self::Balance;

	// TODO: reserve
	// fn extend_lock();

	/// Remove an existing lock.
	///
	/// The function will return the sum of expired locks' amount.
	fn remove_locks(who: &AccountId, lock: &Self::Lock) -> Self::Balance;
}

pub trait Locks {
	type Balance;
	type Lock;
	type Moment;
	type WithdrawReasons;

	/// The number of locks.
	fn locks_count(&self) -> u32;

	/// - Create a new balance lock on account `who`.
	/// 	- If the new lock is valid (i.e. not already expired), it will push the struct to
	/// 	the `Locks` vec in storage. Note that you can lock more funds than a user has.
	/// 	- If the lock `id/until` already exists, this will update it.
	/// - Remove the expired locks on account `who`.
	/// - Update the global staking amount.
	/// - The function will return the sum of expired locks' amount.
	fn update_lock(&mut self, lock: Self::Lock, at: Self::Moment) -> Self::Balance;

	/// Remove expired locks
	fn remove_expired_locks(&mut self, at: Self::Moment) -> Self::Balance;

	/// Remove specify locks and expired locks
	fn remove_locks(&mut self, lock: &Self::Lock, at: Self::Moment) -> Self::Balance;

	fn can_withdraw(&self, at: Self::Moment, reasons: Self::WithdrawReasons, new_balance: Self::Balance) -> bool;
}
