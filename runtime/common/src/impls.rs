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

// --- core ---
use core::marker::PhantomData;
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
