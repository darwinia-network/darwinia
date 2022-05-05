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

// --- crates.io ---
use codec::{Decode, Encode};
use scale_info::TypeInfo;
// --- paritytech ---
use frame_support::{
	weights::{DispatchClass, Weight},
	RuntimeDebug,
};
use sp_runtime::{traits::Zero, FixedPointNumber, FixedU128};
use sp_std::{convert::TryFrom, ops::RangeInclusive};
// --- darwinia-network ---
use crate::*;
use bp_message_dispatch::CallOrigin;
use bp_messages::{source_chain::*, target_chain::*, *};
use bp_runtime::{messages::*, ChainId, *};
use bridge_runtime_common::{
	lanes::*,
	messages::{
		self,
		source::{self, *},
		target::{self, *},
		BalanceOf, *,
	},
};
use darwinia_common_runtime::impls::FromThisChainMessageVerifier;
use dp_s2s::{CallParams, CreatePayload};
use pallet_bridge_messages::EXPECTED_DEFAULT_MESSAGE_LENGTH;

/// Messages delivery proof for Crab -> Darwinia messages.
type ToDarwiniaMessagesDeliveryProof = FromBridgedChainMessagesDeliveryProof<bp_darwinia::Hash>;
/// Messages proof for Darwinia -> Crab messages.
type FromDarwiniaMessagesProof = FromBridgedChainMessagesProof<bp_darwinia::Hash>;

/// Message payload for Crab -> Darwinia messages.
pub type ToDarwiniaMessagePayload = FromThisChainMessagePayload<WithDarwiniaMessageBridge>;
/// Message payload for Darwinia -> Crab messages.
pub type FromDarwiniaMessagePayload = FromBridgedChainMessagePayload<WithDarwiniaMessageBridge>;

/// Message verifier for Crab -> Darwinia messages.
pub type ToDarwiniaMessageVerifier =
	FromThisChainMessageVerifier<WithDarwiniaMessageBridge, Runtime, WithDarwiniaFeeMarket>;

/// Encoded Crab Call as it comes from Darwinia.
pub type FromDarwiniaEncodedCall = FromBridgedChainEncodedMessageCall<Call>;

/// Call-dispatch based message dispatch for Darwinia -> Crab messages.
pub type FromDarwiniaMessageDispatch =
	FromBridgedChainMessageDispatch<WithDarwiniaMessageBridge, Runtime, Ring, WithDarwiniaDispatch>;

/// The s2s backing pallet index in the darwinia chain runtime.
pub const DARWINIA_S2S_BACKING_PALLET_INDEX: u8 = 46;

/// Initial value of `DarwiniaToCrabConversionRate` parameter.
pub const INITIAL_DARWINIA_TO_CRAB_CONVERSION_RATE: FixedU128 =
	FixedU128::from_inner(FixedU128::DIV);

frame_support::parameter_types! {
	/// Darwinia to Crab conversion rate. Initially we treat both tokens as equal.
	pub storage DarwiniaToCrabConversionRate: FixedU128 = INITIAL_DARWINIA_TO_CRAB_CONVERSION_RATE;
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ToDarwiniaOutboundPayLoad;
impl CreatePayload<bp_crab::AccountId, bp_crab::AccountPublic, bp_crab::Signature>
	for ToDarwiniaOutboundPayLoad
{
	type Payload = ToDarwiniaMessagePayload;

	fn create(
		origin: CallOrigin<bp_crab::AccountId, bp_crab::AccountPublic, bp_crab::Signature>,
		spec_version: u32,
		weight: u64,
		call_params: CallParams,
		dispatch_fee_payment: DispatchFeePayment,
	) -> Result<Self::Payload, &'static str> {
		let call = Self::encode_call(DARWINIA_S2S_BACKING_PALLET_INDEX, call_params)?;
		Ok(ToDarwiniaMessagePayload {
			spec_version,
			weight,
			origin,
			call,
			dispatch_fee_payment,
		})
	}
}

/// Crab -> Darwinia message lane pallet parameters.
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum CrabToDarwiniaMessagesParameter {
	/// The conversion formula we use is: `CrabTokens = DarwiniaTokens * conversion_rate`.
	DarwiniaToCrabConversionRate(FixedU128),
}
impl Parameter for CrabToDarwiniaMessagesParameter {
	fn save(&self) {
		match *self {
			CrabToDarwiniaMessagesParameter::DarwiniaToCrabConversionRate(ref conversion_rate) => {
				DarwiniaToCrabConversionRate::set(conversion_rate)
			}
		}
	}
}

/// Darwinia <-> Crab message bridge.
#[derive(Clone, Copy, RuntimeDebug)]
pub struct WithDarwiniaMessageBridge;
impl MessageBridge for WithDarwiniaMessageBridge {
	const RELAYER_FEE_PERCENT: u32 = 10;
	const THIS_CHAIN_ID: ChainId = CRAB_CHAIN_ID;
	const BRIDGED_CHAIN_ID: ChainId = DARWINIA_CHAIN_ID;
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str = bp_crab::WITH_CRAB_MESSAGES_PALLET_NAME;

	type ThisChain = Crab;
	type BridgedChain = Darwinia;

	fn bridged_balance_to_this_balance(
		bridged_balance: BalanceOf<Self::BridgedChain>,
	) -> BalanceOf<Self::ThisChain> {
		<BalanceOf<Self::ThisChain>>::try_from(
			DarwiniaToCrabConversionRate::get().saturating_mul_int(bridged_balance),
		)
		.unwrap_or(<BalanceOf<Self::ThisChain>>::MAX)
	}
}

/// Crab chain from message lane point of view.
#[derive(Clone, Copy, RuntimeDebug)]
pub struct Crab;
impl ChainWithMessages for Crab {
	type Hash = bp_crab::Hash;
	type AccountId = bp_crab::AccountId;
	type Signer = bp_crab::AccountPublic;
	type Signature = bp_crab::Signature;
	type Weight = Weight;
	type Balance = bp_crab::Balance;
}
impl ThisChainWithMessages for Crab {
	type Call = Call;

	fn is_outbound_lane_enabled(lane: &LaneId) -> bool {
		*lane == DARWINIA_CRAB_LANE
	}

	fn maximal_pending_messages_at_outbound_lane() -> MessageNonce {
		MessageNonce::MAX
	}

	fn estimate_delivery_confirmation_transaction() -> MessageTransaction<Weight> {
		let inbound_data_size = InboundLaneData::<Self::AccountId>::encoded_size_hint(
			bp_darwinia::MAXIMAL_ENCODED_ACCOUNT_ID_SIZE,
			1,
			1,
		)
		.unwrap_or(u32::MAX);

		MessageTransaction {
			dispatch_weight: bp_darwinia::MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT,
			size: inbound_data_size
				.saturating_add(bp_darwinia::EXTRA_STORAGE_PROOF_SIZE)
				.saturating_add(bp_darwinia::TX_EXTRA_BYTES),
		}
	}

	fn transaction_payment(transaction: MessageTransaction<Weight>) -> Balance {
		// in our testnets, both per-byte fee and weight-to-fee are 1:1
		messages::transaction_payment(
			bp_crab::RuntimeBlockWeights::get()
				.get(DispatchClass::Normal)
				.base_extrinsic,
			1,
			FixedU128::zero(),
			|weight| weight as _,
			transaction,
		)
	}
}

/// Darwinia chain from message lane point of view.
#[derive(Clone, Copy, RuntimeDebug)]
pub struct Darwinia;
impl ChainWithMessages for Darwinia {
	type Hash = bp_darwinia::Hash;
	type AccountId = bp_darwinia::AccountId;
	type Signer = bp_darwinia::AccountPublic;
	type Signature = bp_darwinia::Signature;
	type Weight = Weight;
	type Balance = bp_darwinia::Balance;
}
impl BridgedChainWithMessages for Darwinia {
	fn maximal_extrinsic_size() -> u32 {
		bp_darwinia::Darwinia::max_extrinsic_size()
	}

	fn message_weight_limits(_message_payload: &[u8]) -> RangeInclusive<Weight> {
		// we don't want to relay too large messages + keep reserve for future upgrades
		let upper_limit = target::maximal_incoming_message_dispatch_weight(
			bp_darwinia::Darwinia::max_extrinsic_weight(),
		);

		// we're charging for payload bytes in `WithDarwiniaMessageBridge::transaction_payment` function
		//
		// this bridge may be used to deliver all kind of messages, so we're not making any assumptions about
		// minimal dispatch weight here

		0..=upper_limit
	}

	fn estimate_delivery_transaction(
		message_payload: &[u8],
		include_pay_dispatch_fee_cost: bool,
		message_dispatch_weight: Weight,
	) -> MessageTransaction<Weight> {
		let message_payload_len = u32::try_from(message_payload.len()).unwrap_or(u32::MAX);
		let extra_bytes_in_payload = Weight::from(message_payload_len)
			.saturating_sub(EXPECTED_DEFAULT_MESSAGE_LENGTH.into());

		MessageTransaction {
			dispatch_weight: extra_bytes_in_payload
				.saturating_mul(bp_darwinia::ADDITIONAL_MESSAGE_BYTE_DELIVERY_WEIGHT)
				.saturating_add(bp_darwinia::DEFAULT_MESSAGE_DELIVERY_TX_WEIGHT)
				.saturating_add(message_dispatch_weight)
				.saturating_sub(if include_pay_dispatch_fee_cost {
					0
				} else {
					bp_darwinia::PAY_INBOUND_DISPATCH_FEE_WEIGHT
				}),
			size: message_payload_len
				.saturating_add(bp_darwinia::EXTRA_STORAGE_PROOF_SIZE)
				.saturating_add(bp_darwinia::TX_EXTRA_BYTES),
		}
	}

	fn transaction_payment(transaction: MessageTransaction<Weight>) -> Self::Balance {
		// in our testnets, both per-byte fee and weight-to-fee are 1:1
		messages::transaction_payment(
			RuntimeBlockWeights::get()
				.get(DispatchClass::Normal)
				.base_extrinsic,
			1,
			FixedU128::zero(),
			|weight| weight as _,
			transaction,
		)
	}
}
impl TargetHeaderChain<ToDarwiniaMessagePayload, <Self as ChainWithMessages>::AccountId>
	for Darwinia
{
	type Error = &'static str;
	// The proof is:
	// - hash of the header this proof has been created with;
	// - the storage proof or one or several keys;
	// - id of the lane we prove state of.
	type MessagesDeliveryProof = ToDarwiniaMessagesDeliveryProof;

	fn verify_message(payload: &ToDarwiniaMessagePayload) -> Result<(), Self::Error> {
		source::verify_chain_message::<WithDarwiniaMessageBridge>(payload)
	}

	fn verify_messages_delivery_proof(
		proof: Self::MessagesDeliveryProof,
	) -> Result<(LaneId, InboundLaneData<bp_crab::AccountId>), Self::Error> {
		source::verify_messages_delivery_proof::<
			WithDarwiniaMessageBridge,
			Runtime,
			WithDarwiniaGrandpa,
		>(proof)
	}
}
impl SourceHeaderChain<<Self as ChainWithMessages>::Balance> for Darwinia {
	type Error = &'static str;
	// The proof is:
	// - hash of the header this proof has been created with;
	// - the storage proof or one or several keys;
	// - id of the lane we prove messages for;
	// - inclusive range of messages nonces that are proved.
	type MessagesProof = FromDarwiniaMessagesProof;

	fn verify_messages_proof(
		proof: Self::MessagesProof,
		messages_count: u32,
	) -> Result<ProvedMessages<Message<<Self as ChainWithMessages>::Balance>>, Self::Error> {
		target::verify_messages_proof::<WithDarwiniaMessageBridge, Runtime, WithDarwiniaGrandpa>(
			proof,
			messages_count,
		)
	}
}
