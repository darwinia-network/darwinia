// --- paritytech ---
use frame_support::PalletId;
// --- darwinia-network ---
use crate::*;
use bp_messages::{LaneId, MessageNonce};
use bp_runtime::{ChainId, DARWINIA_PARACHAIN_CHAIN_ID};
use bridge_runtime_common::lanes::DARWINIA_DARWINIA_PARACHAIN_LANE;
use to_parachain_backing::{Config, LatestMessageNoncer};

// TODO: check
pub const DARWINIA_PARACHAIN_ISSUING_PALLET_INDEX: u8 = 24;

pub struct DarwiniaParachainMessageNoncer;
impl LatestMessageNoncer for DarwiniaParachainMessageNoncer {
	fn outbound_latest_generated_nonce(lane_id: LaneId) -> MessageNonce {
		pallet_bridge_messages::OutboundLanes::<Runtime, WithDarwiniaParachainMessages>::get(&lane_id)
			.latest_generated_nonce
	}

	fn inbound_latest_received_nonce(lane_id: LaneId) -> MessageNonce {
		pallet_bridge_messages::InboundLanes::<Runtime, WithDarwiniaParachainMessages>::get(&lane_id)
			.last_delivered_nonce()
	}
}

frame_support::parameter_types! {
	pub const DarwiniaParachainChainId: ChainId = DARWINIA_PARACHAIN_CHAIN_ID;
	pub const S2sBackingPalletId: PalletId = PalletId(*b"da/s2sba");
	pub const MaxLockRingAmountPerTx: Balance = 1_000_000 * COIN;
	pub const BridgeDarwiniaParachainLaneId: LaneId = DARWINIA_DARWINIA_PARACHAIN_LANE;
}

impl Config for Runtime {
	type BridgedAccountIdConverter = bp_darwinia_parachain::AccountIdConverter;
	type BridgedChainId = DarwiniaParachainChainId;
	type Event = Event;
	type MaxLockRingAmountPerTx = MaxLockRingAmountPerTx;
	type MessageLaneId = BridgeDarwiniaParachainLaneId;
	type MessageNoncer = DarwiniaParachainMessageNoncer;
	type MessagesBridge = BridgeDarwiniaParachainMessages;
	type OutboundPayloadCreator = bm_darwinia_parachain::ToDarwiniaParachainOutboundPayLoad;
	type PalletId = S2sBackingPalletId;
	type RingCurrency = Ring;
	type WeightInfo = ();
}
