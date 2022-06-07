// --- paritytech ---
use frame_support::traits::ChangeMembers;
use pallet_membership::{Config, Instance1};
// --- darwinia-network ---
use crate::*;

pub struct MembershipChangedGroup;
impl ChangeMembers<AccountId> for MembershipChangedGroup {
	fn change_members_sorted(
		incoming: &[AccountId],
		outgoing: &[AccountId],
		sorted_new: &[AccountId],
	) {
		TechnicalCommittee::change_members_sorted(incoming, outgoing, sorted_new);
		EthereumRelay::change_members_sorted(incoming, outgoing, sorted_new);
	}
}

impl Config<Instance1> for Runtime {
	type AddOrigin = RootOrMoreThanHalf<CouncilCollective>;
	type Event = Event;
	type MaxMembers = MaxMembers;
	type MembershipChanged = MembershipChangedGroup;
	type MembershipInitialized = TechnicalCommittee;
	type PrimeOrigin = RootOrMoreThanHalf<CouncilCollective>;
	type RemoveOrigin = RootOrMoreThanHalf<CouncilCollective>;
	type ResetOrigin = RootOrMoreThanHalf<CouncilCollective>;
	type SwapOrigin = RootOrMoreThanHalf<CouncilCollective>;
	type WeightInfo = ();
}
