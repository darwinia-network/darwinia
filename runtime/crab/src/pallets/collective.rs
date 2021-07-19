pub use pallet_collective::{Instance1 as CouncilCollective, Instance2 as TechnicalCollective};

// --- substrate ---
use frame_system::{EnsureOneOf, EnsureRoot};
use pallet_collective::{
	Config, EnsureProportionAtLeast, EnsureProportionMoreThan, PrimeDefaultVote,
};
use sp_core::u32_trait::{_1, _2, _3, _5};
// --- darwinia ---
use crate::{weights::pallet_collective::WeightInfo, *};

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

#[cfg(feature = "dev")]
frame_support::parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 3 * MINUTES;
	pub const TechnicalMotionDuration: BlockNumber = 3 * MINUTES;
}
#[cfg(not(feature = "dev"))]
frame_support::parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 3 * DAYS;
	pub const TechnicalMotionDuration: BlockNumber = 3 * DAYS;
}
frame_support::parameter_types! {
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
	pub const TechnicalMaxProposals: u32 = 100;
	pub const TechnicalMaxMembers: u32 = 100;
}

// Make sure that there are no more than MaxMembers members elected via phragmen.
static_assertions::const_assert!(DesiredMembers::get() <= CouncilMaxMembers::get());

impl Config<CouncilCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = PrimeDefaultVote;
	type WeightInfo = WeightInfo<Runtime>;
}
impl Config<TechnicalCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = TechnicalMaxProposals;
	type MaxMembers = TechnicalMaxMembers;
	type DefaultVote = PrimeDefaultVote;
	type WeightInfo = WeightInfo<Runtime>;
}
