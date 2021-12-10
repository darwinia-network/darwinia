pub use pallet_bridge_dispatch::Instance1 as WithDarwiniaDispatch;

// --- paritytech ---
use bp_messages::{LaneId, MessageNonce};
use frame_support::traits::Contains;
use pallet_bridge_dispatch::Config;
// --- darwinia-network ---
use crate::*;
use darwinia_message::FromDarwiniaEncodedCall;

pub struct S2sCallFilter;
impl Contains<Call> for S2sCallFilter {
	fn contains(c: &Call) -> bool {
		matches!(
			c,
			Call::System(frame_system::Call::remark(_))
				| Call::System(frame_system::Call::remark_with_event(_))
				| Call::Substrate2SubstrateIssuing(
					from_substrate_issuing::Call::register_from_remote(..)
				) | Call::Substrate2SubstrateIssuing(from_substrate_issuing::Call::issue_from_remote(
				..
			))
		)
	}
}

impl Config<WithDarwiniaDispatch> for Runtime {
	type Event = Event;
	type BridgeMessageId = (LaneId, MessageNonce);
	type Call = Call;
	type CallFilter = S2sCallFilter;
	type EncodedCall = FromDarwiniaEncodedCall;
	type SourceChainAccountId = AccountId;
	type TargetChainAccountPublic = AccountPublic;
	type TargetChainSignature = Signature;
	type AccountIdConverter = AccountIdConverter;
}
