// --- paritytech ---
use frame_support::PalletId;
use pallet_babe::RandomnessFromOneEpochAgo;
use pallet_society::{Config, EnsureFounder};
// --- darwinia-network ---
use crate::*;

frame_support::parameter_types! {
	pub const SocietyPalletId: PalletId = PalletId(*b"da/socie");
	pub const CandidateDeposit: Balance = 10 * COIN;
	pub const WrongSideDeduction: Balance = 2 * COIN;
	pub const MaxStrikes: u32 = 10;
	pub const PeriodSpend: Balance = 500 * COIN;
	pub const RotationPeriod: BlockNumber = 80 * HOURS;
	pub const MaxLockDuration: BlockNumber = 36 * 30 * DAYS;
	pub const ChallengePeriod: BlockNumber = 7 * DAYS;
	pub const MaxCandidateIntake: u32 = 1;
}

impl Config for Runtime {
	type Event = Event;
	type PalletId = SocietyPalletId;
	type Currency = Ring;
	type Randomness = RandomnessFromOneEpochAgo<Self>;
	type CandidateDeposit = CandidateDeposit;
	type WrongSideDeduction = WrongSideDeduction;
	type MaxStrikes = MaxStrikes;
	type PeriodSpend = PeriodSpend;
	type MembershipChanged = ();
	type RotationPeriod = RotationPeriod;
	type MaxLockDuration = MaxLockDuration;
	type FounderSetOrigin = EnsureRootOrMoreThanHalfCouncil;
	type SuspensionJudgementOrigin = EnsureFounder<Self>;
	type ChallengePeriod = ChallengePeriod;
	type MaxCandidateIntake = MaxCandidateIntake;
}
