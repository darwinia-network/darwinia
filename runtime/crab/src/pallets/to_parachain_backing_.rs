// --- crates.io ---
use codec::{Decode, Encode};
use scale_info::TypeInfo;
// --- paritytech ---
use frame_support::{PalletId, RuntimeDebug};
// --- darwinia-network ---
use crate::{crab_parachain::*, *};
use bp_message_dispatch::CallOrigin;
use bp_messages::LaneId;
use bp_runtime::{messages::DispatchFeePayment, ChainId, CRAB_PARACHAIN_ID};
use bridge_runtime_common::lanes::CRAB_CRAB_PARACHAIN_LANE;
use darwinia_support::s2s::LatestMessageNoncer;
use to_parachain_backing::{Config, IssueFromRemotePayload, IssuingCall};

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

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ToCrabParachainOutboundPayLoad;
impl
	IssueFromRemotePayload<
		bp_crab::AccountId,
		bp_crab::AccountPublic,
		bp_crab::Signature,
		Runtime,
	> for ToCrabParachainOutboundPayLoad
{
	type Payload = ToCrabParachainOutboundPayLoad;

	fn create(
		origin: CallOrigin<
			bp_crab::AccountId,
			bp_crab::AccountPublic,
			bp_crab::Signature,
		>,
		spec_version: u32,
		weight: u64,
		call_params: IssuingCall<Runtime>,
		dispatch_fee_payment: DispatchFeePayment,
	) -> Result<Self::Payload, &'static str> {
		let mut call = vec![CRAB_PARACHAIN_ISSUING_PALLET_INDEX];
		call.append(&mut call_params.encode());
		Ok(ToCrabParachainOutboundPayLoad {
			spec_version,
			weight,
			origin,
			call,
			dispatch_fee_payment,
		})
	}
}

frame_support::parameter_types! {
	pub const CrabParachainChainId: ChainId = CRAB_PARACHAIN_ID;
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
	type OutboundPayloadCreator = ToCrabParachainOutboundPayLoad;
	type PalletId = S2sBackingPalletId;
	type RingCurrency = Ring;
	type WeightInfo = ();
}
