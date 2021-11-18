// // --- paritytech ---
// use bp_messages::{LaneId, MessageNonce};
// use frame_support::traits::Contains;
// use pallet_bridge_dispatch::Config;
// use sp_runtime::{MultiSignature, MultiSigner};
// // --- darwinia-network ---
// use crate::*;
// use bridge_primitives::AccountIdConverter;
// use pangolin_messages::FromPangolinEncodedCall;

// pub struct S2sCallFilter;
// impl Contains<Call> for S2sCallFilter {
// 	fn contains(call: &Call) -> bool {
// 		// matches!(
// 		// 	*call,
// 		// 	Call::Substrate2SubstrateBacking(to_substrate_backing::Call::unlock_from_remote(..))
// 		// )
// 		todo!()
// 	}
// }

// impl Config<WithPangolinDispatch> for Runtime {
// 	type Event = Event;
// 	type BridgeMessageId = (LaneId, MessageNonce);
// 	type Call = Call;
// 	type CallFilter = S2sCallFilter;
// 	type EncodedCall = FromPangolinEncodedCall;
// 	type SourceChainAccountId = pangolin_primitives::AccountId;
// 	type TargetChainAccountPublic = MultiSigner;
// 	type TargetChainSignature = MultiSignature;
// 	type AccountIdConverter = AccountIdConverter;
// }
