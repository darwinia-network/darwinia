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

// --- core ---
use core::ops::RangeInclusive;
// --- crates.io ---
use codec::{Decode, Encode};
use scale_info::TypeInfo;
// --- paritytech ---
use frame_support::{
	weights::{DispatchClass, Weight},
	RuntimeDebug,
};
use sp_runtime::{traits::Zero, FixedPointNumber, FixedU128};
// --- darwinia-network ---
use crate::*;
use bp_kusama::parachains::ParaId;
use bp_message_dispatch::CallOrigin;
use bp_messages::{source_chain::*, target_chain::*, *};
use bp_runtime::{messages::DispatchFeePayment, ChainId, *};
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
use pallet_bridge_messages::EXPECTED_DEFAULT_MESSAGE_LENGTH;
use to_parachain_backing::{IssueFromRemotePayload, IssuingCall};

/// Message delivery proof for Crab -> CrabParachain messages.
type ToCrabParachainMessagesDeliveryProof =
	FromBridgedChainMessagesDeliveryProof<bp_crab_parachain::Hash>;
/// Message proof for CrabParachain -> Crab  messages.
type FromCrabParachainMessagesProof = FromBridgedChainMessagesProof<bp_crab_parachain::Hash>;

/// Message payload for Crab -> CrabParachain messages.
pub type ToCrabParachainMessagePayload =
	FromThisChainMessagePayload<WithCrabParachainMessageBridge>;
/// Message payload for CrabParachain -> Crab messages.
pub type FromCrabParachainMessagePayload =
	FromBridgedChainMessagePayload<WithCrabParachainMessageBridge>;

/// Message verifier for Crab -> CrabParachain messages.
pub type ToCrabParachainMessageVerifier = FromThisChainMessageVerifier<
	WithCrabParachainMessageBridge,
	Runtime,
	WithCrabParachainFeeMarket,
>;

/// Encoded Crab Call as it comes from CrabParachain
pub type FromCrabParachainEncodedCall = FromBridgedChainEncodedMessageCall<Call>;

/// Call-dispatch based message dispatch for CrabParachain -> Crab messages.
pub type FromCrabParachainMessageDispatch = FromBridgedChainMessageDispatch<
	WithCrabParachainMessageBridge,
	Runtime,
	Ring,
	WithCrabParachainDispatch,
>;

/// Identifier of CrabParachain registered in the kusama relay chain.
pub const CRAB_PARACHAIN_ID: u32 = 2105;

pub const INITIAL_CRAB_PARACHAIN_TO_CRAB_CONVERSION_RATE: FixedU128 =
	FixedU128::from_inner(FixedU128::DIV);

frame_support::parameter_types! {
	/// CrabParachain to Crab conversion rate. Initially we trate both tokens as equal.
	pub storage CrabParachainToCrabConversionRate: FixedU128 = INITIAL_CRAB_PARACHAIN_TO_CRAB_CONVERSION_RATE;
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ToCrabParachainOutboundPayLoad;
impl IssueFromRemotePayload<bp_crab::AccountId, bp_crab::AccountPublic, bp_crab::Signature, Runtime>
	for ToCrabParachainOutboundPayLoad
{
	type Payload = ToCrabParachainMessagePayload;

	fn create(
		origin: CallOrigin<bp_crab::AccountId, bp_crab::AccountPublic, bp_crab::Signature>,
		spec_version: u32,
		weight: u64,
		call_params: IssuingCall<Runtime>,
		dispatch_fee_payment: DispatchFeePayment,
	) -> Result<Self::Payload, &'static str> {
		let mut call = vec![CRAB_PARACHAIN_ISSUING_PALLET_INDEX];
		call.append(&mut call_params.encode());
		Ok(Self::Payload { spec_version, weight, origin, call, dispatch_fee_payment })
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum CrabToCrabParachainParameter {
	/// The conversion formula we use is: `CrabTokens = CrabParachainTokens *
	/// conversion_rate`.
	CrabParachainToCrabConversionRate(FixedU128),
}
impl Parameter for CrabToCrabParachainParameter {
	fn save(&self) {
		match *self {
			CrabToCrabParachainParameter::CrabParachainToCrabConversionRate(
				ref conversion_rate,
			) => CrabParachainToCrabConversionRate::set(conversion_rate),
		}
	}
}

/// Crab <-> CrabParachain message bridge.
#[derive(Clone, Copy, RuntimeDebug)]
pub struct WithCrabParachainMessageBridge;
impl MessageBridge for WithCrabParachainMessageBridge {
	type BridgedChain = CrabParachain;
	type ThisChain = Crab;

	const BRIDGED_CHAIN_ID: ChainId = CRAB_PARACHAIN_CHAIN_ID;
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str = bp_crab::WITH_CRAB_MESSAGES_PALLET_NAME;
	const RELAYER_FEE_PERCENT: u32 = 10;
	const THIS_CHAIN_ID: ChainId = CRAB_CHAIN_ID;

	fn bridged_balance_to_this_balance(
		bridged_balance: BalanceOf<Self::BridgedChain>,
	) -> BalanceOf<Self::ThisChain> {
		<BalanceOf<Self::ThisChain>>::try_from(
			CrabParachainToCrabConversionRate::get().saturating_mul_int(bridged_balance),
		)
		.unwrap_or(<BalanceOf<Self::ThisChain>>::MAX)
	}
}

/// Crab chain from message lane point of view.
#[derive(Clone, Copy, RuntimeDebug)]
pub struct Crab;
impl ChainWithMessages for Crab {
	type AccountId = bp_crab::AccountId;
	type Balance = bp_crab::Balance;
	type Hash = bp_crab::Hash;
	type Signature = bp_crab::Signature;
	type Signer = bp_crab::AccountPublic;
	type Weight = Weight;
}
impl ThisChainWithMessages for Crab {
	type Call = Call;

	fn is_outbound_lane_enabled(lane: &LaneId) -> bool {
		*lane == CRAB_CRAB_PARACHAIN_LANE
	}

	fn maximal_pending_messages_at_outbound_lane() -> MessageNonce {
		MessageNonce::MAX
	}

	fn estimate_delivery_confirmation_transaction() -> MessageTransaction<Weight> {
		let inbound_data_size = InboundLaneData::<Self::AccountId>::encoded_size_hint(
			bp_crab::MAXIMAL_ENCODED_ACCOUNT_ID_SIZE,
			1,
			1,
		)
		.unwrap_or(u32::MAX);

		MessageTransaction {
			dispatch_weight: bp_crab::MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT,
			size: inbound_data_size
				.saturating_add(bp_crab::EXTRA_STORAGE_PROOF_SIZE)
				.saturating_add(bp_crab::TX_EXTRA_BYTES),
		}
	}

	fn transaction_payment(transaction: MessageTransaction<Weight>) -> Self::Balance {
		// in our testnets, both per-byte fee and weight-to-fee are 1:1
		messages::transaction_payment(
			RuntimeBlockWeights::get().get(DispatchClass::Normal).base_extrinsic,
			1,
			FixedU128::zero(),
			|weight| weight as _,
			transaction,
		)
	}
}

#[derive(Clone, Copy, RuntimeDebug)]
pub struct CrabParachain;
impl ChainWithMessages for CrabParachain {
	type AccountId = bp_crab_parachain::AccountId;
	type Balance = bp_crab_parachain::Balance;
	type Hash = bp_crab_parachain::Hash;
	type Signature = bp_crab_parachain::Signature;
	type Signer = bp_crab_parachain::AccountPublic;
	type Weight = Weight;
}
impl BridgedChainWithMessages for CrabParachain {
	fn maximal_extrinsic_size() -> u32 {
		bp_crab_parachain::CrabParachain::max_extrinsic_size()
	}

	fn message_weight_limits(_message_payload: &[u8]) -> RangeInclusive<Self::Weight> {
		let upper_limit = target::maximal_incoming_message_dispatch_weight(
			bp_crab_parachain::CrabParachain::max_extrinsic_weight(),
		);
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
				.saturating_mul(bp_crab::ADDITIONAL_MESSAGE_BYTE_DELIVERY_WEIGHT)
				.saturating_add(bp_crab::DEFAULT_MESSAGE_DELIVERY_TX_WEIGHT)
				.saturating_add(message_dispatch_weight)
				.saturating_sub(if include_pay_dispatch_fee_cost {
					0
				} else {
					bp_crab::PAY_INBOUND_DISPATCH_FEE_WEIGHT
				}),
			size: message_payload_len
				.saturating_add(bp_crab::EXTRA_STORAGE_PROOF_SIZE)
				.saturating_add(bp_crab::TX_EXTRA_BYTES),
		}
	}

	fn transaction_payment(transaction: MessageTransaction<Weight>) -> Self::Balance {
		// in our testnets, both per-byte fee and weight-to-fee are 1:1
		messages::transaction_payment(
			bp_crab_parachain::RuntimeBlockWeights::get().get(DispatchClass::Normal).base_extrinsic,
			1,
			FixedU128::zero(),
			|weight| weight as _,
			transaction,
		)
	}
}
impl TargetHeaderChain<ToCrabParachainMessagePayload, <Self as ChainWithMessages>::AccountId>
	for CrabParachain
{
	type Error = &'static str;
	type MessagesDeliveryProof = ToCrabParachainMessagesDeliveryProof;

	fn verify_message(payload: &ToCrabParachainMessagePayload) -> Result<(), Self::Error> {
		source::verify_chain_message::<WithCrabParachainMessageBridge>(payload)
	}

	fn verify_messages_delivery_proof(
		proof: Self::MessagesDeliveryProof,
	) -> Result<(LaneId, InboundLaneData<bp_crab::AccountId>), Self::Error> {
		source::verify_messages_delivery_proof_from_parachain::<
			WithCrabParachainMessageBridge,
			bp_crab_parachain::Header,
			Runtime,
			WithKusamaParachainsInstance,
		>(ParaId(CRAB_PARACHAIN_ID), proof)
	}
}
impl SourceHeaderChain<<Self as ChainWithMessages>::Balance> for CrabParachain {
	type Error = &'static str;
	type MessagesProof = FromCrabParachainMessagesProof;

	fn verify_messages_proof(
		proof: Self::MessagesProof,
		messages_count: u32,
	) -> Result<ProvedMessages<Message<<Self as ChainWithMessages>::Balance>>, Self::Error> {
		target::verify_messages_proof_from_parachain::<
			WithCrabParachainMessageBridge,
			bp_crab_parachain::Header,
			Runtime,
			WithKusamaParachainsInstance,
		>(ParaId(CRAB_PARACHAIN_ID), proof, messages_count)
	}
}
