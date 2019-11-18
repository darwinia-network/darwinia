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
	/// The quantity used to denote time; usually just a `BlockNumber`.
	/// In Darwinia we prefer using `TimeStamp/u64`.
	type Moment;
	/// Customize our `WithdrawReasons`
	type WithdrawReasons;

	/// Recommend to define `Id` as below and customize `PartialEq` to differentiate the locks:
	/// ```rust
	/// #[derive(Clone, Default, Eq, Encode, Decode, RuntimeDebug)]
	/// pub struct Lock<Balance, Moment> {
	/// 	pub amount: Balance,
	/// 	pub until: Moment,
	/// 	pub reasons: WithdrawReasons,
	/// }
	/// ```
	/// Create a new balance lock on account `who`.
	///
	/// If the new lock is valid (i.e. not already expired), it will push the struct to
	/// the `Locks` vec in storage. Note that you can lock more funds than a user has.
	///
	/// If the lock `id/until` already exists, this will update it.
	///
	/// The function will return the sum of expired locks' amount.
	fn set_lock(
		who: &AccountId,
		amount: Self::Balance,
		at: Self::Moment,
		reasons: Self::WithdrawReasons,
	) -> Self::Balance;

	// TODO: reserve
	// fn extend_lock();

	/// Remove an existing lock.
	///
	/// The function will return the sum of expired locks' amount.
	fn remove_lock(who: &AccountId, at: Self::Moment) -> Self::Balance;

	/// The number of locks.
	fn locks_count(who: &AccountId) -> u32;
}
