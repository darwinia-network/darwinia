pub use pallet_bridge_dispatch::Instance1 as WithDarwiniaDispatch;

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
				| Call::FromDarwiniaIssuing(
					from_substrate_issuing::Call::register_from_remote { .. }
				) | Call::FromDarwiniaIssuing(from_substrate_issuing::Call::issue_from_remote { .. })
		)
	}
}

impl Config<WithDarwiniaDispatch> for Runtime {
	type AccountIdConverter = bp_crab::AccountIdConverter;
	type BridgeMessageId = (LaneId, MessageNonce);
	type Call = Call;
	type CallFilter = S2sCallFilter;
	type EncodedCall = bm_darwinia::FromDarwiniaEncodedCall;
	type Event = Event;
	type SourceChainAccountId = bp_darwinia::AccountId;
	type TargetChainAccountPublic = bp_crab::AccountPublic;
	type TargetChainSignature = bp_crab::Signature;
}
