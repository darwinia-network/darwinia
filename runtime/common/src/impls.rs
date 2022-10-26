// This file is part of Darwinia.
//
// Copyright (C) 2018-2022 Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

//! Auxillary struct/enums for Darwinia runtime.

// --- crates.io ---
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
// --- paritytech ---
use frame_support::traits::{Currency, Imbalance, OnUnbalanced};
use sp_runtime::{traits::TrailingZeroInput, RuntimeDebug};
// --- darwinia-network ---
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
	R: darwinia_balances::Config<RingInstance> + pallet_authorship::Config,
{
	fn on_nonzero_unbalanced(amount: NegativeImbalance<R>) {
		if let Some(author) = <pallet_authorship::Pallet<R>>::author() {
			<darwinia_balances::Pallet<R, RingInstance>>::resolve_creating(&author, amount);
		}
	}
}

pub struct DealWithFees<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for DealWithFees<R>
where
	R: darwinia_balances::Config<RingInstance>
		+ pallet_treasury::Config
		+ pallet_authorship::Config,
	pallet_treasury::Pallet<R>: OnUnbalanced<NegativeImbalance<R>>,
	<R as frame_system::Config>::AccountId:
		From<darwinia_primitives::AccountId> + Into<darwinia_primitives::AccountId>,
	<R as frame_system::Config>::Event: From<darwinia_balances::Event<R, RingInstance>>,
{
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance<R>>) {
		if let Some(fees) = fees_then_tips.next() {
			// for fees, 80% to treasury, 20% to author
			let mut split = fees.ration(80, 20);

			if let Some(tips) = fees_then_tips.next() {
				// for tips, if any, 100% to author
				tips.merge_into(&mut split.1);
			}

			<pallet_treasury::Pallet<R> as OnUnbalanced<_>>::on_unbalanced(split.0);
			<ToAuthor<R> as OnUnbalanced<_>>::on_unbalanced(split.1);
		}
	}
}

/// A source of random balance for the NPoS Solver, which is meant to be run by the OCW election
/// miner.
pub struct OffchainRandomBalancing;
impl frame_support::pallet_prelude::Get<Option<(usize, sp_npos_elections::ExtendedBalance)>>
	for OffchainRandomBalancing
{
	fn get() -> Option<(usize, sp_npos_elections::ExtendedBalance)> {
		let iters = match MINER_MAX_ITERATIONS {
			0 => 0,
			max => {
				let seed = sp_io::offchain::random_seed();
				let random = <u32>::decode(&mut TrailingZeroInput::new(&seed))
					.expect("input is padded with zeroes; qed")
					% max.saturating_add(1);
				random as usize
			},
		};

		Some((iters, 0))
	}
}

#[macro_export]
macro_rules! impl_self_contained_call {
	() => {
		impl fp_self_contained::SelfContainedCall for Call {
			type SignedInfo = H160;

			fn is_self_contained(&self) -> bool {
				match self {
					Call::Ethereum(call) => call.is_self_contained(),
					_ => false,
				}
			}

			fn check_self_contained(
				&self,
			) -> Option<
				Result<
					Self::SignedInfo,
					sp_runtime::transaction_validity::TransactionValidityError,
				>,
			> {
				match self {
					Call::Ethereum(call) => call.check_self_contained(),
					_ => None,
				}
			}

			fn validate_self_contained(
				&self,
				info: &Self::SignedInfo,
			) -> Option<sp_runtime::transaction_validity::TransactionValidity> {
				match self {
					Call::Ethereum(ref call) =>
						Some(validate_self_contained_inner(&self, &call, info)),
					_ => None,
				}
			}

			fn pre_dispatch_self_contained(
				&self,
				info: &Self::SignedInfo,
			) -> Option<Result<(), sp_runtime::transaction_validity::TransactionValidityError>> {
				match self {
					Call::Ethereum(call) => call.pre_dispatch_self_contained(info),
					_ => None,
				}
			}

			fn apply_self_contained(
				self,
				info: Self::SignedInfo,
			) -> Option<sp_runtime::DispatchResultWithInfo<PostDispatchInfoOf<Self>>> {
				match self {
					call @ Call::Ethereum(darwinia_ethereum::Call::transact { .. }) =>
						Some(call.dispatch(Origin::from(
							darwinia_ethereum::RawOrigin::EthereumTransaction(info),
						))),
					_ => None,
				}
			}
		}

		fn validate_self_contained_inner(
			call: &Call,
			eth_call: &darwinia_ethereum::Call<Runtime>,
			signed_info: &<Call as fp_self_contained::SelfContainedCall>::SignedInfo,
		) -> sp_runtime::transaction_validity::TransactionValidity {
			if let darwinia_ethereum::Call::transact { ref transaction } = eth_call {
				// Previously, ethereum transactions were contained in an unsigned
				// extrinsic, we now use a new form of dedicated extrinsic defined by
				// frontier, but to keep the same behavior as before, we must perform
				// the controls that were performed on the unsigned extrinsic.
				use sp_runtime::traits::SignedExtension as _;
				let input_len = match transaction {
					darwinia_ethereum::Transaction::Legacy(t) => t.input.len(),
					darwinia_ethereum::Transaction::EIP2930(t) => t.input.len(),
					darwinia_ethereum::Transaction::EIP1559(t) => t.input.len(),
				};
				let extra_validation =
					SignedExtra::validate_unsigned(call, &call.get_dispatch_info(), input_len)?;
				// Then, do the controls defined by the ethereum pallet.
				use fp_self_contained::SelfContainedCall as _;
				let self_contained_validation =
					eth_call.validate_self_contained(signed_info).ok_or(
						sp_runtime::transaction_validity::TransactionValidityError::Invalid(
							sp_runtime::transaction_validity::InvalidTransaction::BadProof,
						),
					)??;

				Ok(extra_validation.combine_with(self_contained_validation))
			} else {
				Err(sp_runtime::transaction_validity::TransactionValidityError::Unknown(
					sp_runtime::transaction_validity::UnknownTransaction::CannotLookup,
				))
			}
		}
	};
}
