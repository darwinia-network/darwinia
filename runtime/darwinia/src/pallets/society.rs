// --- substrate ---
use frame_support::PalletId;
use pallet_society::{Config, EnsureFounder};
// --- darwinia ---
use crate::*;

#[cfg(feature = "dev")]
frame_support::parameter_types! {
	pub const RotationPeriod: BlockNumber = 3 * MINUTES;
	pub const MaxLockDuration: BlockNumber = 3 * MINUTES;
	pub const ChallengePeriod: BlockNumber = 3 * MINUTES;
}
#[cfg(not(feature = "dev"))]
frame_support::parameter_types! {
	pub const RotationPeriod: BlockNumber = 80 * HOURS;
	pub const MaxLockDuration: BlockNumber = 36 * 30 * DAYS;
	pub const ChallengePeriod: BlockNumber = 7 * DAYS;
}
frame_support::parameter_types! {
	pub const SocietyPalletId: PalletId = PalletId(*b"da/socie");
	pub const CandidateDeposit: Balance = 10 * COIN;
	pub const WrongSideDeduction: Balance = 2 * COIN;
	pub const MaxStrikes: u32 = 10;
	pub const PeriodSpend: Balance = 500 * COIN;
	pub const MaxCandidateIntake: u32 = 1;
}

impl Config for Runtime {
	type Event = Event;
	type PalletId = SocietyPalletId;
	type Currency = Ring;
	type Randomness = RandomnessCollectiveFlip;
	type CandidateDeposit = CandidateDeposit;
	type WrongSideDeduction = WrongSideDeduction;
	type MaxStrikes = MaxStrikes;
	type PeriodSpend = PeriodSpend;
	type MembershipChanged = ();
	type RotationPeriod = RotationPeriod;
	type MaxLockDuration = MaxLockDuration;
	type FounderSetOrigin = EnsureRootOrMoreThanHalfCouncil;
	type SuspensionJudgementOrigin = EnsureFounder<Runtime>;
	type ChallengePeriod = ChallengePeriod;
	type MaxCandidateIntake = MaxCandidateIntake;
}
