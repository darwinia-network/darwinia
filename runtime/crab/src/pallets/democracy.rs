// --- paritytech ---
use frame_system::{EnsureOneOf, EnsureRoot};
use pallet_collective::{EnsureMember, EnsureProportionAtLeast};
use sp_core::u32_trait::{_1, _2, _3};
// --- darwinia-network ---
use crate::*;
use darwinia_democracy::Config;

frame_support::parameter_types! {
	pub const EnactmentPeriod: BlockNumber = 8 * DAYS;
	pub const LaunchPeriod: BlockNumber = 7 * DAYS;
	pub const VotingPeriod: BlockNumber = 7 * DAYS;
	pub const MinimumDeposit: Balance = 1 * MILLI;
	pub const InstantAllowed: bool = true;
	pub const FastTrackVotingPeriod: BlockNumber = 3 * HOURS;
	pub const CooloffPeriod: BlockNumber = 7 * DAYS;
	pub const PreimageByteDeposit: Balance = 10 * NANO;
	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = 100;
}

impl Config for Runtime {
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>,
	>;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin = EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollective>;
	type CooloffPeriod = CooloffPeriod;
	type Currency = Ring;
	type EnactmentPeriod = EnactmentPeriod;
	type Event = Event;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin = EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>;
	/// A majority can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin = EnsureRootOrHalfCouncil;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin = EnsureRootOrHalfCouncil;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin = EnsureProportionAtLeast<_2, _3, AccountId, TechnicalCollective>;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	type InstantAllowed = InstantAllowed;
	type InstantOrigin = EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>;
	type LaunchPeriod = LaunchPeriod;
	type MaxProposals = MaxProposals;
	type MaxVotes = MaxVotes;
	type MinimumDeposit = MinimumDeposit;
	type OperationalPreimageOrigin = EnsureMember<AccountId, CouncilCollective>;
	type PalletsOrigin = OriginCaller;
	type PreimageByteDeposit = PreimageByteDeposit;
	type Proposal = Call;
	type Scheduler = Scheduler;
	type Slash = Treasury;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cool-off period.
	type VetoOrigin = EnsureMember<AccountId, TechnicalCollective>;
	type VoteLockingPeriod = EnactmentPeriod;
	type VotingPeriod = VotingPeriod;
	type WeightInfo = ();
}
