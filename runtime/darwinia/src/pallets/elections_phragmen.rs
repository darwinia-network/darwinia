// --- substrate ---
use frame_support::traits::{LockIdentifier, U128CurrencyToVote};
// --- darwinia ---
use crate::{weights::darwinia_elections_phragmen::WeightInfo, *};
use darwinia_elections_phragmen::Config;

frame_support::parameter_types! {
	pub const PhragmenElectionPalletId: LockIdentifier = *b"phrelect";
	pub const CandidacyBond: Balance = 100 * MILLI;
	// 1 storage item created, key size is 32 bytes, value size is 16+16.
	pub const VotingBondBase: Balance = deposit(1, 64);
	// additional data per vote is 32 bytes (account id).
	pub const VotingBondFactor: Balance = deposit(0, 32);
	/// Daily council elections.
	pub const TermDuration: BlockNumber = 7 * DAYS;
	pub const DesiredMembers: u32 = 7;
	pub const DesiredRunnersUp: u32 = 7;
}

impl Config for Runtime {
	type Event = Event;
	type PalletId = PhragmenElectionPalletId;
	type Currency = Ring;
	type ChangeMembers = Council;
	type InitializeMembers = Council;
	type CurrencyToVote = U128CurrencyToVote;
	type CandidacyBond = CandidacyBond;
	type VotingBondBase = VotingBondBase;
	type VotingBondFactor = VotingBondFactor;
	type LoserCandidate = Treasury;
	type KickedMember = Treasury;
	type DesiredMembers = DesiredMembers;
	type DesiredRunnersUp = DesiredRunnersUp;
	type TermDuration = TermDuration;
	type WeightInfo = WeightInfo<Runtime>;
}
