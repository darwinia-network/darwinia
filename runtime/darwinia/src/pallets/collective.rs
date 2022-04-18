pub use pallet_collective::{Instance1 as CouncilCollective, Instance2 as TechnicalCollective};

// --- paritytech ---
use frame_system::{EnsureOneOf, EnsureRoot};
use pallet_collective::{
	Config, EnsureProportionAtLeast, EnsureProportionMoreThan, PrimeDefaultVote,
};
use sp_core::u32_trait::{_1, _2, _3, _5};
// --- darwinia-network ---
use crate::*;

pub type EnsureRootOrHalfCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>,
>;
pub type EnsureRootOrMoreThanHalfCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>,
>;
pub type EnsureRootOrHalfTechnicalComittee = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	EnsureProportionMoreThan<_1, _2, AccountId, TechnicalCollective>,
>;

pub type ApproveOrigin = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	EnsureProportionAtLeast<_3, _5, AccountId, CouncilCollective>,
>;
pub type TechnicalCommitteeApproveOrigin = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	EnsureProportionMoreThan<_3, _5, AccountId, TechnicalCollective>,
>;

frame_support::parameter_types! {
	pub const MotionDuration: BlockNumber = 7 * DAYS;
	pub const MaxProposals: u32 = 100;
	pub const MaxMembers: u32 = 100;
}

// Make sure that there are no more than MaxMembers members elected via phragmen.
static_assertions::const_assert!(DesiredMembers::get() <= MaxMembers::get());

impl Config<CouncilCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = MotionDuration;
	type MaxProposals = MaxProposals;
	type MaxMembers = MaxMembers;
	type DefaultVote = PrimeDefaultVote;
	type WeightInfo = ();
}
impl Config<TechnicalCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = MotionDuration;
	type MaxProposals = MaxProposals;
	type MaxMembers = MaxMembers;
	type DefaultVote = PrimeDefaultVote;
	type WeightInfo = ();
}
