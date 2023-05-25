// This file is part of Darwinia.
//
// Copyright (C) 2018-2023 Darwinia Network
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

// paritytech
use frame_support::{weights::Weight, RuntimeDebug};
// darwinia
use crate::*;
use bp_messages::{source_chain::*, target_chain::*, *};
use bp_polkadot_core::parachains::ParaId;
use bp_runtime::*;
use bridge_runtime_common::{
	lanes::*,
	messages::{source::*, target::*, *},
};
use darwinia_common_runtime::*;

/// Message delivery proof for Pangoro -> Pangolin messages.
pub type ToPangolinMessagesDeliveryProof = FromBridgedChainMessagesDeliveryProof<bp_pangolin::Hash>;
/// Message proof for Pangolin -> Pangoro messages.
pub type FromPangolinMessagesProof = FromBridgedChainMessagesProof<bp_pangolin::Hash>;

/// Message payload for Pangoro -> Pangolin messages.
pub type ToPangolinMessagePayload = FromThisChainMessagePayload<WithPangolinMessageBridge>;
/// Message payload for Pangolin -> Pangoro messages.
pub type FromPangolinMessagePayload = FromBridgedChainMessagePayload<WithPangolinMessageBridge>;

/// Message verifier for Pangoro -> Pangolin messages.
pub type ToPangolinMessageVerifier<R> =
	FromThisChainMessageVerifier<WithPangolinMessageBridge, R, WithPangolinFeeMarket>;

/// Encoded Pangolin Call as it comes from Pangolin.
pub type FromPangolinEncodedCall = FromBridgedChainEncodedMessageCall<RuntimeCall>;

/// Call-dispatch based message dispatch for Pangolin -> Pangoro messages.
pub type FromPangolinMessageDispatch = FromBridgedChainMessageDispatch<
	WithPangolinMessageBridge,
	Runtime,
	Balances,
	WithPangolinDispatch,
>;

/// Maximal size of message payload to Pangolin chain.
pub type ToPangolinMaximalOutboundPayloadSize =
	bridge_runtime_common::messages::source::FromThisChainMaximalOutboundPayloadSize<
		WithPangolinMessageBridge,
	>;

/// Pangolin <-> Pangoro message bridge.
#[derive(Clone, Copy, RuntimeDebug)]
pub struct WithPangolinMessageBridge;
impl MessageBridge for WithPangolinMessageBridge {
	type BridgedChain = Pangolin;
	type ThisChain = Pangoro;

	const BRIDGED_CHAIN_ID: bp_runtime::ChainId = PANGOLIN_CHAIN_ID;
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str =
		bridge_runtime_common::pallets::WITH_PANGORO_MESSAGES_PALLET_NAME;
	const THIS_CHAIN_ID: bp_runtime::ChainId = PANGORO_CHAIN_ID;
}

#[derive(Clone, Copy, RuntimeDebug)]
pub struct Pangoro;
impl ChainWithMessages for Pangoro {
	type AccountId = bp_pangoro::AccountId;
	type Balance = bp_pangoro::Balance;
	type Hash = bp_pangoro::Hash;
	type Signature = bp_pangoro::Signature;
	type Signer = bp_pangoro::AccountPublic;
}
impl ThisChainWithMessages for Pangoro {
	type RuntimeCall = RuntimeCall;
	type RuntimeOrigin = RuntimeOrigin;

	fn is_message_accepted(_send_origin: &Self::RuntimeOrigin, lane: &LaneId) -> bool {
		*lane == PANGORO_PANGOLIN_LANE
	}

	fn maximal_pending_messages_at_outbound_lane() -> MessageNonce {
		MessageNonce::MAX
	}
}

#[derive(Clone, Copy, RuntimeDebug)]
pub struct Pangolin;
impl ChainWithMessages for Pangolin {
	type AccountId = bp_pangolin::AccountId;
	type Balance = bp_pangolin::Balance;
	type Hash = bp_pangolin::Hash;
	type Signature = bp_pangolin::Signature;
	type Signer = bp_pangolin::AccountPublic;
}
impl BridgedChainWithMessages for Pangolin {
	fn maximal_extrinsic_size() -> u32 {
		bp_pangolin::DarwiniaLike::max_extrinsic_size()
	}

	fn verify_dispatch_weight(_message_payload: &[u8], payload_weight: &Weight) -> bool {
		let upper_limit = target::maximal_incoming_message_dispatch_weight(
			bp_pangolin::DarwiniaLike::max_extrinsic_weight(),
		);
		payload_weight.all_lte(upper_limit)
	}
}
impl TargetHeaderChain<ToPangolinMessagePayload, <Self as ChainWithMessages>::AccountId>
	for Pangolin
{
	type MessagesDeliveryProof = ToPangolinMessagesDeliveryProof;

	fn verify_message(
		payload: &ToPangolinMessagePayload,
	) -> Result<(), bp_messages::VerificationError> {
		source::verify_chain_message::<WithPangolinMessageBridge>(payload)
	}

	fn verify_messages_delivery_proof(
		proof: Self::MessagesDeliveryProof,
	) -> Result<(LaneId, InboundLaneData<bp_pangolin::AccountId>), bp_messages::VerificationError> {
		#[cfg(feature = "runtime-benchmarks")]
		return source::verify_messages_delivery_proof::<
			WithPangolinMessageBridge,
			Runtime,
			WithRococoParachainsInstance,
		>(proof);
		#[cfg(not(feature = "runtime-benchmarks"))]
		source::verify_messages_delivery_proof_from_parachain::<
			WithPangolinMessageBridge,
			bp_pangolin::Header,
			Runtime,
			WithRococoParachainsInstance,
		>(ParaId(2105), proof)
	}
}
impl SourceHeaderChain<<Self as ChainWithMessages>::Balance> for Pangolin {
	type MessagesProof = FromPangolinMessagesProof;

	fn verify_messages_proof(
		proof: Self::MessagesProof,
		messages_count: u32,
	) -> Result<
		ProvedMessages<Message<<Self as ChainWithMessages>::Balance>>,
		bp_messages::VerificationError,
	> {
		#[cfg(feature = "runtime-benchmarks")]
		return target::verify_messages_proof::<
			WithPangolinMessageBridge,
			Runtime,
			WithRococoParachainsInstance,
		>(proof, messages_count);
		#[cfg(not(feature = "runtime-benchmarks"))]
		target::verify_messages_proof_from_parachain::<
			WithPangolinMessageBridge,
			bp_pangolin::Header,
			Runtime,
			WithRococoParachainsInstance,
		>(ParaId(2105), proof, messages_count)
	}
}
