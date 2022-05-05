pub use pallet_bridge_dispatch::Instance1 as WithCrabDispatch;

// --- paritytech ---
use frame_support::traits::Contains;
// --- darwinia-network ---
use crate::*;
use bp_messages::{LaneId, MessageNonce};
use pallet_bridge_dispatch::Config;

pub struct S2sCallFilter;
impl Contains<Call> for S2sCallFilter {
	fn contains(c: &Call) -> bool {
		matches!(
			c,
			Call::System(frame_system::Call::remark { .. })
				| Call::System(frame_system::Call::remark_with_event { .. })
				| Call::ToCrabBacking(to_substrate_backing::Call::unlock_from_remote { .. })
		)
	}
}

impl Config<WithCrabDispatch> for Runtime {
	type AccountIdConverter = bp_darwinia::AccountIdConverter;
	type BridgeMessageId = (LaneId, MessageNonce);
	type Call = Call;
	type CallFilter = S2sCallFilter;
	type EncodedCall = bm_crab::FromCrabEncodedCall;
	type Event = Event;
	type SourceChainAccountId = bp_crab::AccountId;
	type TargetChainAccountPublic = bp_darwinia::AccountPublic;
	type TargetChainSignature = bp_darwinia::Signature;
}
