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
use frame_support::{weights::Weight, RuntimeDebug};
use sp_runtime::{FixedPointNumber, FixedU128};
// --- darwinia-network ---
use crate::*;
use bp_messages::{source_chain::*, target_chain::*, *};
use bp_polkadot::parachains::ParaId;
use bp_runtime::{ChainId, *};
use bridge_runtime_common::{
	lanes::*,
	messages::{
		source::{self, *},
		target::{self, *},
		*,
	},
};

/// Message delivery proof for Darwinia -> DarwiniaParachain messages.
type ToDarwiniaParachainMessagesDeliveryProof =
	FromBridgedChainMessagesDeliveryProof<bp_darwinia_parachain::Hash>;
/// Message proof for DarwiniaParachain -> Darwinia  messages.
type FromDarwiniaParachainMessagesProof =
	FromBridgedChainMessagesProof<bp_darwinia_parachain::Hash>;

/// Message payload for Darwinia -> DarwiniaParachain messages.
pub type ToDarwiniaParachainMessagePayload =
	FromThisChainMessagePayload<WithDarwiniaParachainMessageBridge>;
/// Message payload for DarwiniaParachain -> Darwinia messages.
pub type FromDarwiniaParachainMessagePayload =
	FromBridgedChainMessagePayload<WithDarwiniaParachainMessageBridge>;

/// Message verifier for Darwinia -> DarwiniaParachain messages.
pub type ToDarwiniaParachainMessageVerifier = FromThisChainMessageVerifier<
	WithDarwiniaParachainMessageBridge,
	Runtime,
	WithDarwiniaParachainFeeMarket,
>;

/// Encoded Darwinia Call as it comes from DarwiniaParachain
pub type FromDarwiniaParachainEncodedCall = FromBridgedChainEncodedMessageCall<Call>;

/// Call-dispatch based message dispatch for DarwiniaParachain -> Darwinia messages.
pub type FromDarwiniaParachainMessageDispatch = FromBridgedChainMessageDispatch<
	WithDarwiniaParachainMessageBridge,
	Runtime,
	Ring,
	WithDarwiniaParachainDispatch,
>;

/// Identifier of DarwiniaParachain registered in the Polkadot relay chain.
pub const DARWINIA_PARACHAIN_ID: u32 = 2105;

pub const INITIAL_DARWINIA_PARACHAIN_TO_DARWINIA_CONVERSION_RATE: FixedU128 =
	FixedU128::from_inner(FixedU128::DIV);

frame_support::parameter_types! {
	/// DarwiniaParachain to Darwinia conversion rate. Initially we treat both tokens as equal.
	pub storage DarwiniaParachainToDarwiniaConversionRate: FixedU128 = INITIAL_DARWINIA_PARACHAIN_TO_DARWINIA_CONVERSION_RATE;
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum DarwiniaToDarwiniaParachainMessageParameter {
	/// The conversion formula we use is: `DarwiniaTokens = DarwiniaParachainTokens *
	/// conversion_rate`.
	DarwiniaParachainToDarwiniaConversionRate(FixedU128),
}
impl Parameter for DarwiniaToDarwiniaParachainMessageParameter {
	fn save(&self) {
		match *self {
			DarwiniaToDarwiniaParachainMessageParameter::DarwiniaParachainToDarwiniaConversionRate(
				ref conversion_rate,
			) => DarwiniaParachainToDarwiniaConversionRate::set(conversion_rate),
		}
	}
}

/// Darwinia <-> DarwiniaParachain message bridge.
#[derive(Clone, Copy, RuntimeDebug)]
pub struct WithDarwiniaParachainMessageBridge;
impl MessageBridge for WithDarwiniaParachainMessageBridge {
	type BridgedChain = DarwiniaParachain;
	type ThisChain = Darwinia;

	const BRIDGED_CHAIN_ID: ChainId = DARWINIA_PARACHAIN_CHAIN_ID;
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str =
		bp_darwinia::WITH_DARWINIA_MESSAGES_PALLET_NAME;
	const RELAYER_FEE_PERCENT: u32 = 10;
	const THIS_CHAIN_ID: ChainId = DARWINIA_CHAIN_ID;
}

/// Darwinia chain from message lane point of view.
#[derive(Clone, Copy, RuntimeDebug)]
pub struct Darwinia;
impl ChainWithMessages for Darwinia {
	type AccountId = bp_darwinia::AccountId;
	type Balance = bp_darwinia::Balance;
	type Hash = bp_darwinia::Hash;
	type Signature = bp_darwinia::Signature;
	type Signer = bp_darwinia::AccountPublic;
	type Weight = Weight;
}
impl ThisChainWithMessages for Darwinia {
	type Call = Call;
	type Origin = Origin;

	fn is_message_accepted(_send_origin: &Self::Origin, lane: &LaneId) -> bool {
		*lane == DARWINIA_DARWINIA_PARACHAIN_LANE
	}

	fn maximal_pending_messages_at_outbound_lane() -> MessageNonce {
		MessageNonce::MAX
	}
}

#[derive(Clone, Copy, RuntimeDebug)]
pub struct DarwiniaParachain;
impl ChainWithMessages for DarwiniaParachain {
	type AccountId = bp_darwinia_parachain::AccountId;
	type Balance = bp_darwinia_parachain::Balance;
	type Hash = bp_darwinia_parachain::Hash;
	type Signature = bp_darwinia_parachain::Signature;
	type Signer = bp_darwinia_parachain::AccountPublic;
	type Weight = Weight;
}
impl BridgedChainWithMessages for DarwiniaParachain {
	fn maximal_extrinsic_size() -> u32 {
		bp_darwinia_parachain::DarwiniaParachain::max_extrinsic_size()
	}

	fn message_weight_limits(_message_payload: &[u8]) -> RangeInclusive<Self::Weight> {
		let upper_limit = target::maximal_incoming_message_dispatch_weight(
			bp_darwinia_parachain::DarwiniaParachain::max_extrinsic_weight(),
		);
		0..=upper_limit
	}
}
impl TargetHeaderChain<ToDarwiniaParachainMessagePayload, <Self as ChainWithMessages>::AccountId>
	for DarwiniaParachain
{
	type Error = &'static str;
	type MessagesDeliveryProof = ToDarwiniaParachainMessagesDeliveryProof;

	fn verify_message(payload: &ToDarwiniaParachainMessagePayload) -> Result<(), Self::Error> {
		source::verify_chain_message::<WithDarwiniaParachainMessageBridge>(payload)
	}

	fn verify_messages_delivery_proof(
		proof: Self::MessagesDeliveryProof,
	) -> Result<(LaneId, InboundLaneData<bp_darwinia::AccountId>), Self::Error> {
		source::verify_messages_delivery_proof_from_parachain::<
			WithDarwiniaParachainMessageBridge,
			bp_darwinia_parachain::Header,
			Runtime,
			WithPolkadotParachainsInstance,
		>(ParaId(DARWINIA_PARACHAIN_ID), proof)
	}
}
impl SourceHeaderChain<<Self as ChainWithMessages>::Balance> for DarwiniaParachain {
	type Error = &'static str;
	type MessagesProof = FromDarwiniaParachainMessagesProof;

	fn verify_messages_proof(
		proof: Self::MessagesProof,
		messages_count: u32,
	) -> Result<ProvedMessages<Message<<Self as ChainWithMessages>::Balance>>, Self::Error> {
		target::verify_messages_proof_from_parachain::<
			WithDarwiniaParachainMessageBridge,
			bp_darwinia_parachain::Header,
			Runtime,
			WithPolkadotParachainsInstance,
		>(ParaId(DARWINIA_PARACHAIN_ID), proof, messages_count)
	}
}
