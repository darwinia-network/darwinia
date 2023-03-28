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

use pallet_bridge_messages::Instance1 as WithPangolinMessages;

// darwinia
use crate::*;

impl bp_messages::source_chain::SenderOrigin<AccountId> for RuntimeOrigin {
	fn linked_account(&self) -> Option<AccountId> {
		match self.caller {
			OriginCaller::system(frame_system::RawOrigin::Signed(ref submitter)) =>
				Some(*submitter),
			_ => None,
		}
	}
}

frame_support::parameter_types! {
	pub const BridgedChainId: bp_runtime::ChainId = bp_runtime::PANGOLIN_CHAIN_ID;
	pub const MaxUnconfirmedMessagesAtInboundLane: bp_messages::MessageNonce =
	darwinia_common_runtime::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	pub const MaxUnrewardedRelayerEntriesAtInboundLane: bp_messages::MessageNonce =
	darwinia_common_runtime::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	pub RootAccountForPayments: Option<AccountId> = None;
}

impl pallet_bridge_messages::Config<WithPangolinMessages> for Runtime {
	type AccountIdConverter = darwinia_common_runtime::AccountIdConverter;
	type BridgedChainId = BridgedChainId;
	type InboundMessageFee = dc_primitives::Balance;
	type InboundPayload = bm_pangolin::FromPangolinMessagePayload;
	type InboundRelayer = dc_primitives::AccountId;
	type LaneMessageVerifier = bm_pangolin::ToPangolinMessageVerifier<Self>;
	type MaxMessagesToPruneAtOnce = ConstU64<8>;
	type MaxUnconfirmedMessagesAtInboundLane = MaxUnconfirmedMessagesAtInboundLane;
	type MaxUnrewardedRelayerEntriesAtInboundLane = MaxUnrewardedRelayerEntriesAtInboundLane;
	type MaximalOutboundPayloadSize = bm_pangolin::ToPangolinMaximalOutboundPayloadSize;
	type MessageDeliveryAndDispatchPayment =
		pallet_fee_market::s2s::FeeMarketPayment<Self, WithPangolinFeeMarket, Balances>;
	type MessageDispatch = bm_pangolin::FromPangolinMessageDispatch;
	type OnDeliveryConfirmed =
		pallet_fee_market::s2s::FeeMarketMessageConfirmedHandler<Self, WithPangolinFeeMarket>;
	type OnMessageAccepted =
		pallet_fee_market::s2s::FeeMarketMessageAcceptedHandler<Self, WithPangolinFeeMarket>;
	type OutboundMessageFee = dc_primitives::Balance;
	type OutboundPayload = bm_pangolin::ToPangolinMessagePayload;
	type Parameter = bm_pangolin::PangoroToPangolinParameter;
	type RuntimeEvent = RuntimeEvent;
	type SourceHeaderChain = bm_pangolin::Pangolin;
	type TargetHeaderChain = bm_pangolin::Pangolin;
	type WeightInfo = ();
}
