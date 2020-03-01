// Copyright 2017-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! # Treasury Module
//!
//! The Treasury module provides a "pot" of funds that can be managed by stakeholders in the
//! system and a structure for making spending proposals from this pot.
//!
//! - [`treasury::Trait`](./trait.Trait.html)
//! - [`Call`](./enum.Call.html)
//!
//! ## Overview
//!
//! The Treasury Module itself provides the pot to store funds, and a means for stakeholders to
//! propose, approve, and deny expenditures.  The chain will need to provide a method (e.g.
//! inflation, fees) for collecting funds.
//!
//! By way of example, the Council could vote to fund the Treasury with a portion of the block
//! reward and use the funds to pay developers.
//!
//! ### Terminology
//!
//! - **Proposal:** A suggestion to allocate funds from the pot to a beneficiary.
//! - **Beneficiary:** An account who will receive the funds from a proposal iff
//! the proposal is approved.
//! - **Deposit:** Funds that a proposer must lock when making a proposal. The
//! deposit will be returned or slashed if the proposal is approved or rejected
//! respectively.
//! - **Pot:** Unspent funds accumulated by the treasury module.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `propose_spend` - Make a spending proposal and stake the required deposit.
//! - `set_pot` - Set the spendable balance of funds.
//! - `configure` - Configure the module's proposal requirements.
//! - `reject_proposal` - Reject a proposal, slashing the deposit.
//! - `approve_proposal` - Accept the proposal, returning the deposit.
//!
//! ## GenesisConfig
//!
//! The Treasury module depends on the [`GenesisConfig`](./struct.GenesisConfig.html).
#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(test)]
mod tests;
mod types {
	use crate::*;

	pub type RingBalance<T> = <RingCurrency<T> as Currency<AccountId<T>>>::Balance;
	pub type RingPositiveImbalance<T> = <RingCurrency<T> as Currency<AccountId<T>>>::PositiveImbalance;
	pub type RingNegativeImbalance<T> = <RingCurrency<T> as Currency<AccountId<T>>>::NegativeImbalance;
	pub type KtonBalance<T> = <KtonCurrency<T> as Currency<AccountId<T>>>::Balance;
	pub type KtonPositiveImbalance<T> = <KtonCurrency<T> as Currency<AccountId<T>>>::PositiveImbalance;
	pub type KtonNegativeImbalance<T> = <KtonCurrency<T> as Currency<AccountId<T>>>::NegativeImbalance;
	pub type StakingBalanceT<T> = StakingBalance<RingBalance<T>, KtonBalance<T>>;

	type AccountId<T> = <T as system::Trait>::AccountId;
	type RingCurrency<T> = <T as Trait>::RingCurrency;
	type KtonCurrency<T> = <T as Trait>::KtonCurrency;
}

use codec::{Decode, Encode};
use darwinia_support::OnUnbalancedKton;
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, ensure, print,
	traits::{Currency, ExistenceRequirement, Get, Imbalance, OnUnbalanced, ReservableCurrency, WithdrawReason},
	weights::SimpleDispatchInfo,
};
use frame_system::{self as system, ensure_signed};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{AccountIdConversion, EnsureOrigin, Saturating, StaticLookup, Zero},
	ModuleId, Permill, RuntimeDebug,
};
use sp_std::prelude::*;
use types::*;

const MODULE_ID: ModuleId = ModuleId(*b"py/trsry");

pub trait Trait: frame_system::Trait {
	/// The staking *RING*.
	type RingCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

	/// The staking *Kton*.
	type KtonCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

	/// Origin from which approvals must come.
	type ApproveOrigin: EnsureOrigin<Self::Origin>;

	/// Origin from which rejections must come.
	type RejectOrigin: EnsureOrigin<Self::Origin>;

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// Handler for the unbalanced decrease when slashing for a rejected proposal.
	type RingProposalRejection: OnUnbalanced<RingNegativeImbalance<Self>>;
	type KtonProposalRejection: OnUnbalanced<KtonNegativeImbalance<Self>>;

	/// Fraction of a proposal's value that should be bonded in order to place the proposal.
	/// An accepted proposal gets these back. A rejected proposal does not.
	type ProposalBond: Get<Permill>;

	/// Minimum amount of funds that should be placed in a deposit for making a proposal.
	type RingProposalBondMinimum: Get<RingBalance<Self>>;
	type KtonProposalBondMinimum: Get<KtonBalance<Self>>;

	/// Period between successive spends.
	type SpendPeriod: Get<Self::BlockNumber>;

	/// Percentage of spare funds (if any) that are burnt per spend period.
	type Burn: Get<Permill>;
}

type ProposalIndex = u32;

/// Reversed for manually usage
#[allow(unused)]
enum Funds {
	Kton,
	Ring,
}

/// To unify *Ring* and *Kton* balances. Ref to the solution at
/// [`darwinia_staking`](../darwinia_staking/enum.StakingBalance.html),
/// keep the `StakingBalance` name for upgrade usages.
#[derive(Copy, Clone, Encode, Decode, Eq, Ord, RuntimeDebug, PartialEq, PartialOrd)]
pub enum StakingBalance<RingBalance, KtonBalance> {
	RingBalance(RingBalance),
	KtonBalance(KtonBalance),
}

impl<RingBalance, KtonBalance> StakingBalance<RingBalance, KtonBalance> {
	fn ring(v: RingBalance) -> Self {
		StakingBalance::RingBalance(v)
	}

	fn kton(v: KtonBalance) -> Self {
		StakingBalance::KtonBalance(v)
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// Fraction of a proposal's value that should be bonded in order to place the proposal.
		/// An accepted proposal gets these back. A rejected proposal does not.
		const ProposalBond: Permill = T::ProposalBond::get();

		/// Minimum amount of funds that should be placed in a deposit for making a proposal.
		const KtonProposalBondMinimum: KtonBalance<T> = T::KtonProposalBondMinimum::get();
		const RingProposalBondMinimum: RingBalance<T> = T::RingProposalBondMinimum::get();

		/// Period between successive spends.
		const SpendPeriod: T::BlockNumber = T::SpendPeriod::get();

		/// Percentage of spare funds (if any) that are burnt per spend period.
		const Burn: Permill = T::Burn::get();

		type Error = Error<T>;

		fn deposit_event() = default;

		/// Put forward a suggestion for spending. A deposit proportional to the value
		/// is reserved and slashed if the proposal is rejected. It is returned once the
		/// proposal is awarded.
		///
		/// # <weight>
		/// - O(1).
		/// - Limited storage reads.
		/// - One DB change, one extra DB entry.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(500_000)]
		fn propose_spend(
			origin,
			value: StakingBalanceT<T>,
			beneficiary: <T::Lookup as StaticLookup>::Source
		) {
			let proposer = ensure_signed(origin)?;
			let beneficiary = T::Lookup::lookup(beneficiary)?;
			let bond = Self::calculate_bond(value);
			match bond {
				StakingBalance::KtonBalance(bond) => {
					T::KtonCurrency::reserve(&proposer, bond)
						.map_err(|_| <Error<T>>::InsufficientProposersBalance)?;
				},
				StakingBalance::RingBalance(bond) => {
					T::RingCurrency::reserve(&proposer, bond)
						.map_err(|_| <Error<T>>::InsufficientProposersBalance)?;
				}
			}

			let c = Self::proposal_count();
			ProposalCount::put(c + 1);
			<Proposals<T>>::insert(c, Proposal {
				proposer,
				beneficiary,
				bond,
				value,
			});

			Self::deposit_event(RawEvent::Proposed(c));
		}

		/// Reject a proposed spend. The original deposit will be slashed.
		///
		/// # <weight>
		/// - O(1).
		/// - Limited storage reads.
		/// - One DB clear.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedOperational(100_000)]
		fn reject_proposal(origin, #[compact] proposal_id: ProposalIndex) {
			T::RejectOrigin::ensure_origin(origin)?;
			let proposal = <Proposals<T>>::take(&proposal_id).ok_or(<Error<T>>::InvalidProposalIndex)?;

			let value = proposal.bond;
			match value {
				StakingBalance::KtonBalance(value) => {
					let imbalance = T::KtonCurrency::slash_reserved(&proposal.proposer, value).0;
					T::KtonProposalRejection::on_unbalanced(imbalance);
				},
				StakingBalance::RingBalance(value) => {
					let imbalance = T::RingCurrency::slash_reserved(&proposal.proposer, value).0;
					T::RingProposalRejection::on_unbalanced(imbalance);
				}
			}

			Self::deposit_event(Event::<T>::Rejected(proposal_id, value));
		}

		/// Approve a proposal. At a later time, the proposal will be allocated to the beneficiary
		/// and the original deposit will be returned.
		///
		/// # <weight>
		/// - O(1).
		/// - Limited storage reads.
		/// - One DB change.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedOperational(100_000)]
		fn approve_proposal(origin, #[compact] proposal_id: ProposalIndex) {
			T::ApproveOrigin::ensure_origin(origin)?;

			ensure!(<Proposals<T>>::exists(proposal_id), <Error<T>>::InvalidProposalIndex);

			Approvals::mutate(|v| v.push(proposal_id));
		}

		/// This function will implement the `OnFinalize` trait
		fn on_finalize(n: T::BlockNumber) {
			// Check to see if we should spend some funds!
			if (n % T::SpendPeriod::get()).is_zero() {
				Self::spend_funds(Funds::Ring);
				Self::spend_funds(Funds::Kton);
			}
		}
	}
}

/// A spending proposal.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, sp_runtime::RuntimeDebug)]
pub struct Proposal<AccountId, StakingBalance> {
	proposer: AccountId,
	value: StakingBalance,
	beneficiary: AccountId,
	bond: StakingBalance,
}

decl_storage! {
	trait Store for Module<T: Trait> as Treasury {
		/// Number of proposals that have been made.
		ProposalCount get(fn proposal_count): ProposalIndex;

		/// Proposals that have been made.
		Proposals get(fn proposals): map ProposalIndex => Option<Proposal<T::AccountId, StakingBalanceT<T>>>;

		/// Proposal indices that have been approved but not yet awarded.
		Approvals get(fn approvals): Vec<ProposalIndex>;
	}
	add_extra_genesis {
		build(|_config| {
			// Create Treasury account
			let _ = T::RingCurrency::make_free_balance_be(
				&<Module<T>>::account_id(),
				T::RingCurrency::minimum_balance(),
			);

			let _ = T::KtonCurrency::make_free_balance_be(
				&<Module<T>>::account_id(),
				T::KtonCurrency::minimum_balance(),
			);
		});
	}
}

decl_event!(
	/// TODO: Events below needs to replace RingBalance to StakingBalance
	pub enum Event<T>
	where
		<T as frame_system::Trait>::AccountId,
		RingBalance = RingBalance<T>,
		KtonBalance = KtonBalance<T>,
	{
		/// New proposal.
		Proposed(ProposalIndex),
		/// We have ended a spend period and will now allocate funds.
		Spending(StakingBalance<RingBalance, KtonBalance>),
		/// Some funds have been allocated.
		Awarded(ProposalIndex, StakingBalance<RingBalance, KtonBalance>, AccountId),
		/// A proposal was rejected; funds were slashed.
		Rejected(ProposalIndex, StakingBalance<RingBalance, KtonBalance>),
		/// Some of our funds have been burnt.
		Burnt(StakingBalance<RingBalance, KtonBalance>),
		/// Spending has finished; this is the amount that rolls over until next spend.
		Rollover(StakingBalance<RingBalance, KtonBalance>),
		/// Some *Ring* have been deposited.
		DepositRing(RingBalance),
		/// Some *Kton* have been deposited.
		DepositKton(KtonBalance),
	}
);

decl_error! {
	/// Error for the treasury module.
	pub enum Error for Module<T: Trait> {
		/// Proposer's balance is too low.
		InsufficientProposersBalance,
		/// No proposal at that index.
		InvalidProposalIndex,
	}
}

impl<T: Trait> Module<T> {
	// Add public immutables and private mutables.

	/// The account ID of the treasury pot.
	///
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	pub fn account_id() -> T::AccountId {
		MODULE_ID.into_account()
	}

	/// The needed bond for a proposal whose spend is `value`.
	fn calculate_bond(value: StakingBalanceT<T>) -> StakingBalanceT<T> {
		match value {
			StakingBalance::KtonBalance(value) => {
				StakingBalance::kton(T::KtonProposalBondMinimum::get().max(T::ProposalBond::get() * value))
			}
			StakingBalance::RingBalance(value) => {
				StakingBalance::ring(T::RingProposalBondMinimum::get().max(T::ProposalBond::get() * value))
			}
		}
	}

	/// TODO: Need a pre-logic to infer what specific to do with kton or ring.
	fn spend_kton_funds() {
		let mut budget_remaining = Self::pot::<T::KtonCurrency>();
		Self::deposit_event(RawEvent::Spending(StakingBalance::kton(budget_remaining)));

		let mut missed_any = false;
		let mut imbalance = <KtonPositiveImbalance<T>>::zero();
		Approvals::mutate(|v| {
			v.retain(|&index| {
				// Should always be true, but shouldn't panic if false or we're screwed.
				let option_proposal = Self::proposals(index);
				if option_proposal.is_none() {
					return false;
				}

				let p = option_proposal.unwrap();
				match (p.value, p.bond) {
					(StakingBalance::KtonBalance(value), StakingBalance::KtonBalance(bond)) => {
						if value > budget_remaining {
							missed_any = true;
							return true;
						}

						budget_remaining -= value;
						<Proposals<T>>::remove(index);

						// return their deposit.
						let _ = T::KtonCurrency::unreserve(&p.proposer, bond);

						// provide the allocation.
						imbalance.subsume(T::KtonCurrency::deposit_creating(&p.beneficiary, value));

						Self::deposit_event(RawEvent::Awarded(
							index,
							StakingBalance::KtonBalance(value),
							p.beneficiary,
						));
						false
					}
					_ => true,
				}
			});
		});

		if !missed_any {
			// burn some proportion of the remaining budget if we run a surplus.
			let burn = (T::Burn::get() * budget_remaining).min(budget_remaining);
			budget_remaining -= burn;
			imbalance.subsume(T::KtonCurrency::burn(burn));
			Self::deposit_event(RawEvent::Burnt(StakingBalance::KtonBalance(burn)))
		}

		// Must never be an error, but better to be safe.
		// proof: budget_remaining is account free balance minus ED;
		// Thus we can't spend more than account free balance minus ED;
		// Thus account is kept alive; qed;
		if let Err(problem) = T::KtonCurrency::settle(
			&Self::account_id(),
			imbalance,
			WithdrawReason::Transfer.into(),
			ExistenceRequirement::KeepAlive,
		) {
			print("Inconsistent state - couldn't settle imbalance for funds spent by treasury");
			// Nothing else to do here.
			drop(problem);
		}

		Self::deposit_event(RawEvent::Rollover(StakingBalance::KtonBalance(budget_remaining)));
	}

	fn spend_ring_funds() {
		let mut budget_remaining = Self::pot::<T::RingCurrency>();
		Self::deposit_event(RawEvent::Spending(StakingBalance::RingBalance(budget_remaining)));

		let mut missed_any = false;
		let mut imbalance = <RingPositiveImbalance<T>>::zero();
		Approvals::mutate(|v| {
			v.retain(|&index| {
				// Should always be true, but shouldn't panic if false or we're screwed.
				let option_proposal = Self::proposals(index);
				if option_proposal.is_none() {
					return false;
				}

				let p = option_proposal.unwrap();
				match (p.value, p.bond) {
					(StakingBalance::RingBalance(value), StakingBalance::RingBalance(bond)) => {
						if value > budget_remaining {
							missed_any = true;
							return true;
						}

						budget_remaining -= value;
						<Proposals<T>>::remove(index);

						// return their deposit.
						let _ = T::RingCurrency::unreserve(&p.proposer, bond);

						// provide the allocation.
						imbalance.subsume(T::RingCurrency::deposit_creating(&p.beneficiary, value));

						Self::deposit_event(RawEvent::Awarded(
							index,
							StakingBalance::RingBalance(value),
							p.beneficiary,
						));
						false
					}
					_ => true,
				}
			});
		});

		if !missed_any {
			// burn some proportion of the remaining budget if we run a surplus.
			let burn = (T::Burn::get() * budget_remaining).min(budget_remaining);
			budget_remaining -= burn;
			imbalance.subsume(T::RingCurrency::burn(burn));
			Self::deposit_event(RawEvent::Burnt(StakingBalance::RingBalance(burn)))
		}

		// Must never be an error, but better to be safe.
		// proof: budget_remaining is account free balance minus ED;
		// Thus we can't spend more than account free balance minus ED;
		// Thus account is kept alive; qed;
		if let Err(problem) = T::RingCurrency::settle(
			&Self::account_id(),
			imbalance,
			WithdrawReason::Transfer.into(),
			ExistenceRequirement::KeepAlive,
		) {
			print("Inconsistent state - couldn't settle imbalance for funds spent by treasury");
			// Nothing else to do here.
			drop(problem);
		}

		Self::deposit_event(RawEvent::Rollover(StakingBalance::RingBalance(budget_remaining)));
	}

	// Spend some money!
	fn spend_funds(fund: Funds) {
		match fund {
			Funds::Kton => Self::spend_kton_funds(),
			Funds::Ring => Self::spend_ring_funds(),
		}
	}

	/// Return the amount of money in the pot.
	// The existential deposit is not part of the pot so treasury account never gets deleted.
	fn pot<C>() -> C::Balance
	where
		C: Currency<T::AccountId>,
	{
		C::free_balance(&Self::account_id())
			// Must never be less than 0 but better be safe.
			.saturating_sub(C::minimum_balance())
	}
}

impl<T: Trait> OnUnbalanced<RingNegativeImbalance<T>> for Module<T> {
	fn on_nonzero_unbalanced(amount: RingNegativeImbalance<T>) {
		let numeric_amount = amount.peek();

		// Must resolve into existing but better to be safe.
		let _ = T::RingCurrency::resolve_creating(&Self::account_id(), amount);

		Self::deposit_event(RawEvent::DepositRing(numeric_amount));
	}
}

// FIXME: Ugly hack due to https://github.com/rust-lang/rust/issues/31844#issuecomment-557918823
impl<T: Trait> OnUnbalancedKton<KtonNegativeImbalance<T>> for Module<T> {
	fn on_nonzero_unbalanced(amount: KtonNegativeImbalance<T>) {
		let numeric_amount = amount.peek();

		// Must resolve into existing but better to be safe.
		let _ = T::KtonCurrency::resolve_creating(&Self::account_id(), amount);

		Self::deposit_event(RawEvent::DepositKton(numeric_amount));
	}
}
