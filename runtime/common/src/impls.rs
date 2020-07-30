//! Auxillary struct/enums for Darwinia runtime.

// --- crates ---
use codec::{Decode, Encode};
// --- substrate ---
use frame_support::traits::{Currency, Imbalance, OnUnbalanced};
use sp_runtime::RuntimeDebug;
// --- darwinia ---
use crate::*;

darwinia_support::impl_account_data! {
	struct AccountData<Balance>
	for
		RingInstance,
		KtonInstance
	where
		Balance = darwinia_primitives::Balance
	{
		// other data
	}
}

/// Logic for the author to get a portion of fees.
pub struct ToAuthor<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for ToAuthor<R>
where
	R: darwinia_balances::Trait<RingInstance> + pallet_authorship::Trait,
	<R as frame_system::Trait>::AccountId: From<darwinia_primitives::AccountId>,
	<R as frame_system::Trait>::AccountId: Into<darwinia_primitives::AccountId>,
	<R as frame_system::Trait>::Event: From<
		darwinia_balances::RawEvent<
			<R as frame_system::Trait>::AccountId,
			<R as darwinia_balances::Trait<RingInstance>>::Balance,
			RingInstance,
		>,
	>,
{
	fn on_nonzero_unbalanced(amount: NegativeImbalance<R>) {
		let numeric_amount = amount.peek();
		let author = <pallet_authorship::Module<R>>::author();
		<darwinia_balances::Module<R, RingInstance>>::resolve_creating(
			&<pallet_authorship::Module<R>>::author(),
			amount,
		);
		<frame_system::Module<R>>::deposit_event(
			<darwinia_balances::RawEvent<_, _, RingInstance>>::Deposit(author, numeric_amount),
		);
	}
}

pub mod support_kton_in_the_future {
	use sp_runtime::traits::Convert;

	use crate::*;

	/// Converter for currencies to votes.
	pub struct CurrencyToVoteHandler<R>(sp_std::marker::PhantomData<R>);
	impl<R> CurrencyToVoteHandler<R>
	where
		R: darwinia_balances::Trait<RingInstance>,
		R::Balance: Into<u128>,
	{
		fn factor() -> u128 {
			let issuance: u128 =
				<darwinia_balances::Module<R, RingInstance>>::total_issuance().into();
			(issuance / u64::max_value() as u128).max(1)
		}
	}
	impl<R> Convert<u128, u64> for CurrencyToVoteHandler<R>
	where
		R: darwinia_balances::Trait<RingInstance>,
		R::Balance: Into<u128>,
	{
		fn convert(x: u128) -> u64 {
			(x / Self::factor()) as u64
		}
	}
	impl<R> Convert<u128, u128> for CurrencyToVoteHandler<R>
	where
		R: darwinia_balances::Trait<RingInstance>,
		R::Balance: Into<u128>,
	{
		fn convert(x: u128) -> u128 {
			x * Self::factor()
		}
	}
}
