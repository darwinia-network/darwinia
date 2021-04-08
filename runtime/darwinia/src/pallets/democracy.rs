// --- substrate ---
use frame_system::{EnsureOneOf, EnsureRoot};
use pallet_collective::{EnsureMember, EnsureProportionAtLeast};
use sp_core::u32_trait::{_1, _2, _3};
// --- darwinia ---
use crate::{weights::darwinia_democracy::WeightInfo, *};
use darwinia_democracy::Config;

frame_support::parameter_types! {
	pub const LaunchPeriod: BlockNumber = 28 * DAYS;
	pub const VotingPeriod: BlockNumber = 28 * DAYS;
	pub const FastTrackVotingPeriod: BlockNumber = 3 * HOURS;
	pub const MinimumDeposit: Balance = 100 * COIN;
	pub const EnactmentPeriod: BlockNumber = 28 * DAYS;
	pub const CooloffPeriod: BlockNumber = 7 * DAYS;
	// One milli: $10,000 / MB
	pub const PreimageByteDeposit: Balance = 1 * MILLI;
	pub const InstantAllowed: bool = true;
	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = 100;
}
impl Config for Runtime {
	type Proposal = Call;
	type Event = Event;
	type Currency = Ring;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type MinimumDeposit = MinimumDeposit;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin = EnsureRootOrHalfCouncil;
	/// A majority can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin = ApproveOrigin;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>,
	>;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		EnsureProportionAtLeast<_2, _3, AccountId, TechnicalCollective>,
	>;
	type InstantOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>,
	>;
	type InstantAllowed = InstantAllowed;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollective>,
	>;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>,
	>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cool-off period.
	type VetoOrigin = EnsureMember<AccountId, TechnicalCollective>;
	type CooloffPeriod = CooloffPeriod;
	type PreimageByteDeposit = PreimageByteDeposit;
	type Slash = Treasury;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = MaxVotes;
	type OperationalPreimageOrigin = EnsureMember<AccountId, CouncilCollective>;
	type MaxProposals = MaxProposals;
	type WeightInfo = WeightInfo<Runtime>;
}
