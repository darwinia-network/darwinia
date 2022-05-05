// --- paritytech ---
use pallet_membership::{Config, Instance1 as TechnicalMembershipInstance};
// --- darwinia-network ---
use crate::*;

impl Config<TechnicalMembershipInstance> for Runtime {
	type AddOrigin = EnsureRootOrMoreThanHalfCouncil;
	type Event = Event;
	type MaxMembers = MaxMembers;
	type MembershipChanged = TechnicalCommittee;
	type MembershipInitialized = TechnicalCommittee;
	type PrimeOrigin = EnsureRootOrMoreThanHalfCouncil;
	type RemoveOrigin = EnsureRootOrMoreThanHalfCouncil;
	type ResetOrigin = EnsureRootOrMoreThanHalfCouncil;
	type SwapOrigin = EnsureRootOrMoreThanHalfCouncil;
	type WeightInfo = ();
}
