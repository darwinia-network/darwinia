// --- paritytech ---
use pallet_membership::{Config, Instance1 as TechnicalMembershipInstance};
// --- darwinia-network ---
use crate::*;

impl Config<TechnicalMembershipInstance> for Runtime {
	type AddOrigin = MoreThanHalf<CouncilCollective>;
	type Event = Event;
	type MaxMembers = MaxMembers;
	type MembershipChanged = TechnicalCommittee;
	type MembershipInitialized = TechnicalCommittee;
	type PrimeOrigin = MoreThanHalf<CouncilCollective>;
	type RemoveOrigin = MoreThanHalf<CouncilCollective>;
	type ResetOrigin = MoreThanHalf<CouncilCollective>;
	type SwapOrigin = MoreThanHalf<CouncilCollective>;
	type WeightInfo = ();
}
