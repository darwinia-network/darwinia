#[cfg(feature = "std")]
pub use darwinia_staking::{Forcing, StakerStatus};

// --- substrate ---
use sp_npos_elections::CompactSolution;
use sp_runtime::ModuleId;
use sp_staking::SessionIndex;
// --- darwinia ---
use crate::{weights::darwinia_staking::WeightInfo, *};
use darwinia_staking::{Config, EraIndex};

frame_support::parameter_types! {
	pub const StakingModuleId: ModuleId = ModuleId(*b"da/staki");
	pub const SessionsPerEra: SessionIndex = SESSIONS_PER_ERA;
	pub const BondingDurationInEra: EraIndex = 14 * DAYS
		/ (SESSIONS_PER_ERA as BlockNumber * BLOCKS_PER_SESSION);
	pub const BondingDurationInBlockNumber: BlockNumber = 14 * DAYS;
	// slightly less than 14 days.
	pub const SlashDeferDuration: EraIndex = 14 * DAYS
		/ (SESSIONS_PER_ERA as BlockNumber * BLOCKS_PER_SESSION) - 1;
	// last 15 minutes of the last session will be for election.
	pub const MaxNominatorRewardedPerValidator: u32 = 64;
	pub const Cap: Balance = CAP;
	pub const TotalPower: Power = TOTAL_POWER;
}
impl Config for Runtime {
	const MAX_NOMINATIONS: u32 = <NposCompactSolution16 as CompactSolution>::LIMIT as u32;
	type Event = Event;
	type ModuleId = StakingModuleId;
	type UnixTime = Timestamp;
	type SessionsPerEra = SessionsPerEra;
	type BondingDurationInEra = BondingDurationInEra;
	type BondingDurationInBlockNumber = BondingDurationInBlockNumber;
	type SlashDeferDuration = SlashDeferDuration;
	/// A super-majority of the council can cancel the slash.
	type SlashCancelOrigin = EnsureRootOrHalfCouncil;
	type SessionInterface = Self;
	type NextNewSession = Session;
	type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
	type ElectionProvider = ElectionProviderMultiPhase;
	type RingCurrency = Ring;
	type RingRewardRemainder = Treasury;
	type RingSlash = Treasury;
	type RingReward = ();
	type KtonCurrency = Kton;
	type KtonSlash = Treasury;
	type KtonReward = ();
	type Cap = Cap;
	type TotalPower = TotalPower;
	type WeightInfo = WeightInfo<Runtime>;
}
