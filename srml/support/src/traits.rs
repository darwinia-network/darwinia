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

//pub trait LockableCurrency<AccountId>: Currency<AccountId> {
//	/// The quantity used to denote time; usually just a `BlockNumber`.
//	type Moment;
//
//	/// Create a new balance lock on account `who`.
//	///
//	/// If the new lock is valid (i.e. not already expired), it will push the struct to
//	/// the `Locks` vec in storage. Note that you can lock more funds than a user has.
//	///
//	/// If the lock `id` already exists, this will update it.
//	fn set_lock(
//		id: LockIdentifier,
//		who: &AccountId,
//		amount: Self::Balance,
//		until: Self::Moment,
//		reasons: WithdrawReasons,
//	);
//
//	/// Changes a balance lock (selected by `id`) so that it becomes less liquid in all
//	/// parameters or creates a new one if it does not exist.
//	///
//	/// Calling `extend_lock` on an existing lock `id` differs from `set_lock` in that it
//	/// applies the most severe constraints of the two, while `set_lock` replaces the lock
//	/// with the new parameters. As in, `extend_lock` will set:
//	/// - maximum `amount`
//	/// - farthest duration (`until`)
//	/// - bitwise mask of all `reasons`
//	fn extend_lock(
//		id: LockIdentifier,
//		who: &AccountId,
//		amount: Self::Balance,
//		until: Self::Moment,
//		reasons: WithdrawReasons,
//	);
//
//	/// Remove an existing lock.
//	fn remove_lock(id: LockIdentifier, who: &AccountId);
//}
