// --- paritytech ---
use frame_support::PalletId;
// --- darwinia-network ---
use crate::*;
use bp_messages::LaneId;
use bp_runtime::{ChainId, CRAB_CHAIN_ID};
use bridge_runtime_common::lanes::DARWINIA_CRAB_LANE;
use darwinia_support::{evm::DeriveEthereumAddress, s2s::LatestMessageNoncer};
use dp_asset::{TokenMetadata, NATIVE_TOKEN_TYPE};
use to_substrate_backing::Config;

pub struct CrabMessageNoncer;
impl LatestMessageNoncer for CrabMessageNoncer {
	fn outbound_latest_generated_nonce(lane_id: LaneId) -> u64 {
		BridgeCrabMessages::outbound_latest_generated_nonce(lane_id).into()
	}

	fn inbound_latest_received_nonce(lane_id: LaneId) -> u64 {
		BridgeCrabMessages::inbound_latest_received_nonce(lane_id).into()
	}
}

frame_support::parameter_types! {
	pub const CrabChainId: ChainId = CRAB_CHAIN_ID;
	pub RingMetadata: TokenMetadata = TokenMetadata::new(
		NATIVE_TOKEN_TYPE,
		PalletId(*b"da/bring").derive_ethereum_address(),
		b"Darwinia Network Native Token".to_vec(),
		b"RING".to_vec(),
		9,
	);
	pub const S2sBackingPalletId: PalletId = PalletId(*b"da/tcrbk");
	pub const MaxLockRingAmountPerTx: Balance = 1_000_000 * COIN;
	pub const BridgeCrabLaneId: LaneId = DARWINIA_CRAB_LANE;
}

impl Config for Runtime {
	type BridgedAccountIdConverter = bp_crab::AccountIdConverter;
	type BridgedChainId = CrabChainId;
	type Event = Event;
	type MaxLockRingAmountPerTx = MaxLockRingAmountPerTx;
	type MessageLaneId = BridgeCrabLaneId;
	type MessageNoncer = CrabMessageNoncer;
	type MessagesBridge = BridgeCrabMessages;
	type OutboundPayloadCreator = bm_crab::ToCrabOutboundPayload;
	type PalletId = S2sBackingPalletId;
	type RingCurrency = Ring;
	type RingMetadata = RingMetadata;
	type WeightInfo = ();
}
