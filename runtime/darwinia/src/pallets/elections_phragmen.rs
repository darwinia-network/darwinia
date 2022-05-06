// --- paritytech ---
use frame_support::traits::{LockIdentifier, U128CurrencyToVote};
use pallet_elections_phragmen::Config;
// --- darwinia-network ---
use crate::*;

frame_support::parameter_types! {
	pub const PhragmenElectionPalletId: LockIdentifier = *b"phrelect";
	pub const CandidacyBond: Balance = 100 * MILLI;
	// 1 storage item created, key size is 32 bytes, value size is 16+16.
	pub const VotingBondBase: Balance = darwinia_deposit(1, 64);
	// additional data per vote is 32 bytes (account id).
	pub const VotingBondFactor: Balance = darwinia_deposit(0, 32);
	/// Daily council elections.
	pub const TermDuration: BlockNumber = 7 * DAYS;
	pub const DesiredMembers: u32 = 7;
	pub const DesiredRunnersUp: u32 = 7;
}

impl Config for Runtime {
	type CandidacyBond = CandidacyBond;
	type ChangeMembers = Council;
	type Currency = Ring;
	type CurrencyToVote = U128CurrencyToVote;
	type DesiredMembers = DesiredMembers;
	type DesiredRunnersUp = DesiredRunnersUp;
	type Event = Event;
	type InitializeMembers = Council;
	type KickedMember = Treasury;
	type LoserCandidate = Treasury;
	type PalletId = PhragmenElectionPalletId;
	type TermDuration = TermDuration;
	type VotingBondBase = VotingBondBase;
	type VotingBondFactor = VotingBondFactor;
	type WeightInfo = ();
}
