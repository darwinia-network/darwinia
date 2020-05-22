//! Auxillary struct/enums for Darwinia runtime.

// --- core ---
use core::num::NonZeroI128;
// --- crates ---
use codec::{Decode, Encode};
// --- substrate ---
use frame_support::traits::{Currency, Get, Imbalance, OnUnbalanced};
use sp_runtime::{
	traits::{Convert, Saturating},
	RuntimeDebug, {Fixed128, Perquintill},
};
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

/// Update the given multiplier based on the following formula
///
///   diff = (previous_block_weight - target_weight)/max_weight
///   v = 0.00004
///   next_weight = weight * (1 + (v * diff) + (v * diff)^2 / 2)
///
/// Where `target_weight` must be given as the `Get` implementation of the `T` generic type.
/// https://research.web3.foundation/en/latest/polkadot/Token%20Economics/#relay-chain-transaction-fees
pub struct TargetedFeeAdjustment<T, R>(sp_std::marker::PhantomData<(T, R)>);

impl<T: Get<Perquintill>, R: frame_system::Trait> Convert<Fixed128, Fixed128>
	for TargetedFeeAdjustment<T, R>
{
	fn convert(multiplier: Fixed128) -> Fixed128 {
		let max_weight = MaximumBlockWeight::get();
		let block_weight = <frame_system::Module<R>>::all_extrinsics_weight()
			.total()
			.min(max_weight);
		let target_weight = (T::get() * max_weight) as u128;
		let block_weight = block_weight as u128;

		// determines if the first_term is positive
		let positive = block_weight >= target_weight;
		let diff_abs = block_weight.max(target_weight) - block_weight.min(target_weight);
		// safe, diff_abs cannot exceed u64 and it can always be computed safely even with the lossy
		// `Fixed128::from_rational`.
		let diff = Fixed128::from_rational(
			diff_abs as i128,
			NonZeroI128::new(max_weight.max(1) as i128).unwrap(),
		);
		let diff_squared = diff.saturating_mul(diff);

		// 0.00004 = 4/100_000 = 40_000/10^9
		let v = Fixed128::from_rational(4, NonZeroI128::new(100_000).unwrap());
		// 0.00004^2 = 16/10^10 Taking the future /2 into account... 8/10^10
		let v_squared_2 = Fixed128::from_rational(8, NonZeroI128::new(10_000_000_000).unwrap());

		let first_term = v.saturating_mul(diff);
		let second_term = v_squared_2.saturating_mul(diff_squared);

		if positive {
			// Note: this is merely bounded by how big the multiplier and the inner value can go,
			// not by any economical reasoning.
			let excess = first_term.saturating_add(second_term);
			multiplier.saturating_add(excess)
		} else {
			// Defensive-only: first_term > second_term. Safe subtraction.
			let negative = first_term.saturating_sub(second_term);
			multiplier
				.saturating_sub(negative)
				// despite the fact that apply_to saturates weight (final fee cannot go below 0)
				// it is crucially important to stop here and don't further reduce the weight fee
				// multiplier. While at -1, it means that the network is so un-congested that all
				// transactions have no weight fee. We stop here and only increase if the network
				// became more busy.
				.max(Fixed128::from_natural(-1))
		}
	}
}
