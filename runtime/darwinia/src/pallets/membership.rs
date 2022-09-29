// --- paritytech ---
use pallet_membership::{Config, Instance1};
// --- darwinia-network ---
use crate::*;

impl Config<Instance1> for Runtime {
	type AddOrigin = RootOrMoreThanHalf<CouncilCollective>;
	type Event = Event;
	type MaxMembers = MaxMembers;
	type MembershipChanged = TechnicalCommittee;
	type MembershipInitialized = TechnicalCommittee;
	type PrimeOrigin = RootOrMoreThanHalf<CouncilCollective>;
	type RemoveOrigin = RootOrMoreThanHalf<CouncilCollective>;
	type ResetOrigin = RootOrMoreThanHalf<CouncilCollective>;
	type SwapOrigin = RootOrMoreThanHalf<CouncilCollective>;
	type WeightInfo = ();
}
