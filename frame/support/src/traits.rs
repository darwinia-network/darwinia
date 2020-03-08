pub use frame_support::traits::{LockIdentifier, VestingSchedule, WithdrawReason, WithdrawReasons};

use frame_support::traits::{Currency, TryDrop};
use sp_runtime::DispatchResult;

use crate::balance::lock::LockFor;

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
		lock_for: LockFor<Self::Balance, Self::Moment>,
		reasons: WithdrawReasons,
	);

	// TODO: for democracy
	// /// Changes a balance lock (selected by `id`) so that it becomes less liquid in all
	// /// parameters or creates a new one if it does not exist.
	// ///
	// /// Calling `extend_lock` on an existing lock `id` differs from `set_lock` in that it
	// /// applies the most severe constraints of the two, while `set_lock` replaces the lock
	// /// with the new parameters. As in, `extend_lock` will set:
	// /// - maximum `amount`
	// /// - bitwise mask of all `reasons`
	// fn extend_lock(
	// 	id: LockIdentifier,
	// 	who: &AccountId,
	// 	amount: Self::Balance,
	// 	reasons: WithdrawReasons,
	// );

	/// Remove an existing lock.
	fn remove_lock(id: LockIdentifier, who: &AccountId);

	/// Get the balance of an account that can be used for transfers, reservations, or any other
	/// non-locking, non-transaction-fee activity. Will be at most `free_balance`.
	fn usable_balance(who: &AccountId) -> Self::Balance;

	/// Get the balance of an account that can be used for paying transaction fees (not tipping,
	/// or any other kind of fees, though). Will be at most `free_balance`.
	fn usable_balance_for_fees(who: &AccountId) -> Self::Balance;
}

pub trait ExistentialCheck<AccountId, Balance> {
	fn try_drop(who: &AccountId) -> (bool, Balance);
}

impl<AccountId, Balance> ExistentialCheck<AccountId, Balance> for ()
where
	Balance: Default,
{
	fn try_drop(_: &AccountId) -> (bool, Balance) {
		// only for tests
		(true, Default::default())
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
