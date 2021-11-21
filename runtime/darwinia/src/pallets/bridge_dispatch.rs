pub use pallet_bridge_dispatch::Instance1 as S2sBridgeDispatch;

// --- paritytech ---
use bp_messages::{LaneId, MessageNonce};
use frame_support::traits::Contains;
use pallet_bridge_dispatch::Config;
// --- darwinia-network ---
use crate::*;
use bridge_primitives::AccountIdConverter;
use crab_messages::FromCrabEncodedCall;

pub struct S2sCallFilter;
impl Contains<Call> for S2sCallFilter {
	fn contains(_c: &Call) -> bool {
		// matches!(
		// 	c,
		// 	Call::Substrate2SubstrateBacking(to_substrate_backing::Call::unlock_from_remote(..))
		// )
		false
	}
}

impl Config<S2sBridgeDispatch> for Runtime {
	type Event = Event;
	type BridgeMessageId = (LaneId, MessageNonce);
	type Call = Call;
	type CallFilter = S2sCallFilter;
	type EncodedCall = FromCrabEncodedCall;
	type SourceChainAccountId = AccountId;
	type TargetChainAccountPublic = AccountPublic;
	type TargetChainSignature = Signature;
	type AccountIdConverter = AccountIdConverter;
}
