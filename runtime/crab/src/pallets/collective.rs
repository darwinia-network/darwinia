pub use pallet_collective::{Instance1 as CouncilCollective, Instance2 as TechnicalCollective};

// --- paritytech ---
use pallet_collective::{Config, PrimeDefaultVote};
// --- darwinia-network ---
use crate::*;

frame_support::parameter_types! {
	pub const MotionDuration: BlockNumber = 3 * DAYS;
	pub const MaxProposals: u32 = 100;
	pub const MaxMembers: u32 = 100;
}

// Make sure that there are no more than MaxMembers members elected via phragmen.
static_assertions::const_assert!(DesiredMembers::get() <= MaxMembers::get());

impl Config<CouncilCollective> for Runtime {
	type DefaultVote = PrimeDefaultVote;
	type Event = Event;
	type MaxMembers = MaxMembers;
	type MaxProposals = MaxProposals;
	type MotionDuration = MotionDuration;
	type Origin = Origin;
	type Proposal = Call;
	type WeightInfo = ();
}
impl Config<TechnicalCollective> for Runtime {
	type DefaultVote = PrimeDefaultVote;
	type Event = Event;
	type MaxMembers = MaxMembers;
	type MaxProposals = MaxProposals;
	type MotionDuration = MotionDuration;
	type Origin = Origin;
	type Proposal = Call;
	type WeightInfo = ();
}
