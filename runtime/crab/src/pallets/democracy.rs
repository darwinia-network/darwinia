// --- paritytech ---
use pallet_collective::EnsureMember;
use pallet_democracy::Config;
// --- darwinia-network ---
use crate::*;

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
	type BlacklistOrigin = Root;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = All<TechnicalCollective>;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin = AtLeastTwoThird<CouncilCollective>;
	type CooloffPeriod = CooloffPeriod;
	type Currency = Ring;
	type EnactmentPeriod = EnactmentPeriod;
	type Event = Event;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin = All<CouncilCollective>;
	/// A majority can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin = AtLeastHalf<CouncilCollective>;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin = AtLeastHalf<CouncilCollective>;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin = AtLeastTwoThird<TechnicalCollective>;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	type InstantAllowed = InstantAllowed;
	type InstantOrigin = All<TechnicalCollective>;
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
