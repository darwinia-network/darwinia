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
use bp_runtime::*;
use bridge_runtime_common::{
	lanes::*,
	messages::{source::*, target::*, *},
};
use darwinia_common_runtime::*;

/// Message delivery proof for Pangolin -> Pangoro messages.
pub type ToPangoroMessagesDeliveryProof = FromBridgedChainMessagesDeliveryProof<bp_pangoro::Hash>;
/// Message proof for Pangoro -> Pangolin messages.
pub type FromPangoroMessagesProof = FromBridgedChainMessagesProof<bp_pangoro::Hash>;

/// Message payload for Pangolin -> Pangoro messages.
pub type ToPangoroMessagePayload = FromThisChainMessagePayload<WithPangoroMessageBridge>;
/// Message payload for Pangoro -> Pangolin messages.
pub type FromPangoroMessagePayload = FromBridgedChainMessagePayload<WithPangoroMessageBridge>;

/// Message verifier for Pangolin -> Pangoro messages.
pub type ToPangoroMessageVerifier<R> =
	FromThisChainMessageVerifier<WithPangoroMessageBridge, R, WithPangoroFeeMarket>;

/// Encoded Pangoro Call as it comes from Pangoro.
pub type FromPangoroEncodedCall = FromBridgedChainEncodedMessageCall<RuntimeCall>;

/// Call-dispatch based message dispatch for Pangoro -> Pangolin messages.
pub type FromPangoroMessageDispatch = FromBridgedChainMessageDispatch<
	WithPangoroMessageBridge,
	Runtime,
	Balances,
	WithPangoroDispatch,
>;

/// Maximal size of message payload to Pangoro chain.
pub type ToPangoroMaximalOutboundPayloadSize =
	bridge_runtime_common::messages::source::FromThisChainMaximalOutboundPayloadSize<
		WithPangoroMessageBridge,
	>;

/// Pangoro <-> Pangolin message bridge.
#[derive(Clone, Copy, RuntimeDebug)]
pub struct WithPangoroMessageBridge;
impl MessageBridge for WithPangoroMessageBridge {
	type BridgedChain = Pangoro;
	type ThisChain = Pangolin;

	const BRIDGED_CHAIN_ID: bp_runtime::ChainId = PANGORO_CHAIN_ID;
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str =
		bridge_runtime_common::pallets::WITH_PANGOLIN_MESSAGES_PALLET_NAME;
	const THIS_CHAIN_ID: bp_runtime::ChainId = PANGOLIN_CHAIN_ID;
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
impl ThisChainWithMessages for Pangolin {
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
pub struct Pangoro;
impl ChainWithMessages for Pangoro {
	type AccountId = bp_pangoro::AccountId;
	type Balance = bp_pangoro::Balance;
	type Hash = bp_pangoro::Hash;
	type Signature = bp_pangoro::Signature;
	type Signer = bp_pangoro::AccountPublic;
}
impl BridgedChainWithMessages for Pangoro {
	fn maximal_extrinsic_size() -> u32 {
		bp_pangoro::DarwiniaLike::max_extrinsic_size()
	}

	fn verify_dispatch_weight(_message_payload: &[u8], payload_weight: &Weight) -> bool {
		let upper_limit = target::maximal_incoming_message_dispatch_weight(
			bp_pangoro::DarwiniaLike::max_extrinsic_weight(),
		);
		payload_weight.all_lte(upper_limit)
	}
}
impl TargetHeaderChain<ToPangoroMessagePayload, <Self as ChainWithMessages>::AccountId>
	for Pangoro
{
	type MessagesDeliveryProof = ToPangoroMessagesDeliveryProof;

	fn verify_message(
		payload: &ToPangoroMessagePayload,
	) -> Result<(), bp_messages::VerificationError> {
		source::verify_chain_message::<WithPangoroMessageBridge>(payload)
	}

	fn verify_messages_delivery_proof(
		proof: Self::MessagesDeliveryProof,
	) -> Result<(LaneId, InboundLaneData<bp_pangoro::AccountId>), bp_messages::VerificationError> {
		#[cfg(feature = "runtime-benchmarks")]
		return source::verify_messages_delivery_proof::<
			WithPangoroMessageBridge,
			Runtime,
			WithMoonbaseParachainsInstance,
		>(proof);
		#[cfg(not(feature = "runtime-benchmarks"))]
		return source::verify_messages_delivery_proof_from_parachain::<
			WithPangoroMessageBridge,
			bp_pangoro::Header,
			Runtime,
			WithMoonbaseParachainsInstance,
		>(ParaId(2105), proof);
	}
}
impl SourceHeaderChain<<Self as ChainWithMessages>::Balance> for Pangoro {
	type MessagesProof = FromPangoroMessagesProof;

	fn verify_messages_proof(
		proof: Self::MessagesProof,
		messages_count: u32,
	) -> Result<
		ProvedMessages<Message<<Self as ChainWithMessages>::Balance>>,
		bp_messages::VerificationError,
	> {
		#[cfg(feature = "runtime-benchmarks")]
		return target::verify_messages_proof::<
			WithPangoroMessageBridge,
			Runtime,
			WithMoonbaseParachainsInstance,
		>(proof, messages_count);
		#[cfg(not(feature = "runtime-benchmarks"))]
		return target::verify_messages_proof_from_parachain::<
			WithPangoroMessageBridge,
			bp_pangoro::Header,
			Runtime,
			WithMoonbaseParachainsInstance,
		>(bp_polkadot_core::parachains::ParaId(2105), proof, messages_count);
	}
}

impl pallet_bridge_messages::WeightInfoExt for weights::MessagesWeightInfo<Runtime> {}
impl pallet_bridge_parachains::WeightInfoExt for crate::weights::ParachainsWeightInfo<Runtime> {}
