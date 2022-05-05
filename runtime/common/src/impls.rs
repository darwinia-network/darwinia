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
use bp_messages::{source_chain::*, *};
use bridge_runtime_common::messages::{source::*, *};

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
	<R as frame_system::Config>::AccountId:
		From<darwinia_primitives::AccountId> + Into<darwinia_primitives::AccountId>,
	<R as frame_system::Config>::Event: From<darwinia_balances::Event<R, RingInstance>>,
{
	fn on_nonzero_unbalanced(amount: NegativeImbalance<R>) {
		let numeric_amount = amount.peek();
		let author = <pallet_authorship::Pallet<R>>::author();
		<darwinia_balances::Pallet<R, RingInstance>>::resolve_creating(
			&<pallet_authorship::Pallet<R>>::author(),
			amount,
		);
		<frame_system::Pallet<R>>::deposit_event(darwinia_balances::Event::Deposit(
			author,
			numeric_amount,
		));
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
			max @ _ => {
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

/// Message verifier that is doing all basic checks.
///
/// This verifier assumes following:
///
/// - all message lanes are equivalent, so all checks are the same;
/// - messages are being dispatched using `pallet-bridge-dispatch` pallet on the target chain.
///
/// Following checks are made:
///
/// - message is rejected if its lane is currently blocked;
/// - message is rejected if there are too many pending (undelivered) messages at the outbound lane;
/// - check that the sender has rights to dispatch the call on target chain using provided dispatch
///   origin;
/// - check that the sender has paid enough funds for both message delivery and dispatch.
#[derive(RuntimeDebug)]
pub struct FromThisChainMessageVerifier<B, R>(PhantomData<(B, R)>);
impl<B, R>
	LaneMessageVerifier<
		AccountIdOf<ThisChain<B>>,
		FromThisChainMessagePayload<B>,
		BalanceOf<ThisChain<B>>,
	> for FromThisChainMessageVerifier<B, R>
where
	B: MessageBridge,
	R: darwinia_fee_market::Config,
	AccountIdOf<ThisChain<B>>: PartialEq + Clone,
	darwinia_fee_market::RingBalance<R>: From<BalanceOf<ThisChain<B>>>,
{
	type Error = &'static str;

	fn verify_message(
		submitter: &Sender<AccountIdOf<ThisChain<B>>>,
		delivery_and_dispatch_fee: &BalanceOf<ThisChain<B>>,
		lane: &LaneId,
		lane_outbound_data: &OutboundLaneData,
		payload: &FromThisChainMessagePayload<B>,
	) -> Result<(), Self::Error> {
		// reject message if lane is blocked
		if !ThisChain::<B>::is_outbound_lane_enabled(lane) {
			return Err(OUTBOUND_LANE_DISABLED);
		}

		// reject message if there are too many pending messages at this lane
		let max_pending_messages = ThisChain::<B>::maximal_pending_messages_at_outbound_lane();
		let pending_messages = lane_outbound_data
			.latest_generated_nonce
			.saturating_sub(lane_outbound_data.latest_received_nonce);
		if pending_messages > max_pending_messages {
			return Err(TOO_MANY_PENDING_MESSAGES);
		}

		// Do the dispatch-specific check. We assume that the target chain uses
		// `Dispatch`, so we verify the message accordingly.
		pallet_bridge_dispatch::verify_message_origin(submitter, payload)
			.map_err(|_| BAD_ORIGIN)?;

		// Do the delivery_and_dispatch_fee. We assume that the delivery and dispatch fee always
		// greater than the fee market provided fee.
		let message_fee: darwinia_fee_market::RingBalance<R> = (*delivery_and_dispatch_fee).into();
		if let Some(market_fee) = darwinia_fee_market::Pallet::<R>::market_fee() {
			// compare with actual fee paid
			if message_fee < market_fee {
				return Err(TOO_LOW_FEE);
			}
		} else {
			const NO_MARKET_FEE: &str = "The fee market are not ready for accepting messages.";

			return Err(NO_MARKET_FEE);
		}

		Ok(())
	}
}
