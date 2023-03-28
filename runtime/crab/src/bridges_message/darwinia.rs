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

/// Message delivery proof for Crab -> Darwinia messages.
pub type ToDarwiniaMessagesDeliveryProof =
	FromBridgedChainMessagesDeliveryProof<dc_primitives::Hash>;
/// Message proof for Darwinia -> Crab messages.
pub type FromDarwiniaMessagesProof = FromBridgedChainMessagesProof<dc_primitives::Hash>;

/// Message payload for Crab -> Darwinia messages.
pub type ToDarwiniaMessagePayload = FromThisChainMessagePayload<WithDarwiniaMessageBridge>;
/// Message payload for Darwinia -> Crab messages.
pub type FromDarwiniaMessagePayload = FromBridgedChainMessagePayload<WithDarwiniaMessageBridge>;

/// Message verifier for Crab -> Darwinia messages.
pub type ToDarwiniaMessageVerifier<R> =
	FromThisChainMessageVerifier<WithDarwiniaMessageBridge, R, WithDarwiniaFeeMarket>;

/// Encoded Darwinia Call as it comes from Darwinia.
pub type FromDarwiniaEncodedCall = FromBridgedChainEncodedMessageCall<RuntimeCall>;

/// Call-dispatch based message dispatch for Darwinia -> Crab messages.
pub type FromDarwiniaMessageDispatch = FromBridgedChainMessageDispatch<
	WithDarwiniaMessageBridge,
	Runtime,
	Balances,
	WithDarwiniaDispatch,
>;

pub type ToDarwiniaMaximalOutboundPayloadSize =
	bridge_runtime_common::messages::source::FromThisChainMaximalOutboundPayloadSize<
		WithDarwiniaMessageBridge,
	>;

/// Darwinia <-> Crab message bridge.
#[derive(Clone, Copy, RuntimeDebug)]
pub struct WithDarwiniaMessageBridge;
impl MessageBridge for WithDarwiniaMessageBridge {
	type BridgedChain = Darwinia;
	type ThisChain = Crab;

	const BRIDGED_CHAIN_ID: bp_runtime::ChainId = DARWINIA_CHAIN_ID;
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str =
		bridge_runtime_common::pallets::WITH_CRAB_MESSAGES_PALLET_NAME;
	const THIS_CHAIN_ID: bp_runtime::ChainId = CRAB_CHAIN_ID;
}

#[derive(Clone, Copy, RuntimeDebug)]
pub struct Crab;
impl ChainWithMessages for Crab {
	type AccountId = dc_primitives::AccountId;
	type Balance = dc_primitives::Balance;
	type Hash = dc_primitives::Hash;
	type Signature = dc_primitives::Signature;
	type Signer = dc_primitives::AccountPublic;
}
impl ThisChainWithMessages for Crab {
	type RuntimeCall = RuntimeCall;
	type RuntimeOrigin = RuntimeOrigin;

	fn is_message_accepted(_send_origin: &Self::RuntimeOrigin, lane: &LaneId) -> bool {
		*lane == DARWINIA_CRAB_LANE
	}

	fn maximal_pending_messages_at_outbound_lane() -> MessageNonce {
		MessageNonce::MAX
	}
}

#[derive(Clone, Copy, RuntimeDebug)]
pub struct Darwinia;
impl ChainWithMessages for Darwinia {
	type AccountId = dc_primitives::AccountId;
	type Balance = dc_primitives::Balance;
	type Hash = dc_primitives::Hash;
	type Signature = dc_primitives::Signature;
	type Signer = dc_primitives::AccountPublic;
}
impl BridgedChainWithMessages for Darwinia {
	fn maximal_extrinsic_size() -> u32 {
		darwinia_common_runtime::DarwiniaLike::max_extrinsic_size()
	}

	fn verify_dispatch_weight(_message_payload: &[u8], payload_weight: &Weight) -> bool {
		let upper_limit = target::maximal_incoming_message_dispatch_weight(
			darwinia_common_runtime::DarwiniaLike::max_extrinsic_weight(),
		);
		payload_weight.all_lte(upper_limit)
	}
}
impl TargetHeaderChain<ToDarwiniaMessagePayload, <Self as ChainWithMessages>::AccountId>
	for Darwinia
{
	type Error = &'static str;
	type MessagesDeliveryProof = ToDarwiniaMessagesDeliveryProof;

	fn verify_message(payload: &ToDarwiniaMessagePayload) -> Result<(), Self::Error> {
		source::verify_chain_message::<WithDarwiniaMessageBridge>(payload)
	}

	fn verify_messages_delivery_proof(
		proof: Self::MessagesDeliveryProof,
	) -> Result<(LaneId, InboundLaneData<dc_primitives::AccountId>), Self::Error> {
		source::verify_messages_delivery_proof_from_parachain::<
			WithDarwiniaMessageBridge,
			dc_primitives::Header,
			Runtime,
			WithPolkadotParachainsInstance,
		>(ParaId(2046), proof)
	}
}
impl SourceHeaderChain<<Self as ChainWithMessages>::Balance> for Darwinia {
	type Error = &'static str;
	type MessagesProof = FromDarwiniaMessagesProof;

	fn verify_messages_proof(
		proof: Self::MessagesProof,
		messages_count: u32,
	) -> Result<ProvedMessages<Message<<Self as ChainWithMessages>::Balance>>, Self::Error> {
		target::verify_messages_proof_from_parachain::<
			WithDarwiniaMessageBridge,
			dc_primitives::Header,
			Runtime,
			WithPolkadotParachainsInstance,
		>(ParaId(2046), proof, messages_count)
	}
}
