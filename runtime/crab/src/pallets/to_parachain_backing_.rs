// --- paritytech ---
use frame_support::PalletId;
// --- darwinia-network ---
use crate::*;
use bp_messages::LaneId;
use bp_runtime::{ChainId, CRAB_PARACHAIN_CHAIN_ID};
use bridge_runtime_common::lanes::CRAB_CRAB_PARACHAIN_LANE;
use darwinia_support::s2s::LatestMessageNoncer;
use to_parachain_backing::Config;

pub const CRAB_PARACHAIN_ISSUING_PALLET_INDEX: u8 = 24;

pub struct CrabParachainMessageNoncer;
impl LatestMessageNoncer for CrabParachainMessageNoncer {
	fn outbound_latest_generated_nonce(lane_id: LaneId) -> u64 {
		BridgeCrabParachainMessages::outbound_latest_generated_nonce(lane_id).into()
	}

	fn inbound_latest_received_nonce(lane_id: LaneId) -> u64 {
		BridgeCrabParachainMessages::inbound_latest_received_nonce(lane_id).into()
	}
}

frame_support::parameter_types! {
	pub const CrabParachainChainId: ChainId = CRAB_PARACHAIN_CHAIN_ID;
	pub const S2sBackingPalletId: PalletId = PalletId(*b"da/s2sba");
	pub const MaxLockRingAmountPerTx: Balance = 1_000_000 * COIN;
	pub const BridgeCrabParachainLaneId: LaneId = CRAB_CRAB_PARACHAIN_LANE;
}

impl Config for Runtime {
	type BridgedAccountIdConverter = bp_crab_parachain::AccountIdConverter;
	type BridgedChainId = CrabParachainChainId;
	type Event = Event;
	type MaxLockRingAmountPerTx = MaxLockRingAmountPerTx;
	type MessageLaneId = BridgeCrabParachainLaneId;
	type MessageNoncer = CrabParachainMessageNoncer;
	type MessagesBridge = BridgeCrabParachainMessages;
	type OutboundPayloadCreator = bm_crab_parachain::ToCrabParachainOutboundPayLoad;
	type PalletId = S2sBackingPalletId;
	type RingCurrency = Ring;
	type WeightInfo = ();
}
