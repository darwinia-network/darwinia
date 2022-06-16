// --- paritytech ---
use frame_support::PalletId;
use sp_npos_elections::NposSolution;
use sp_runtime::Perbill;
use sp_staking::SessionIndex;
// --- darwinia-network ---
use crate::*;
use darwinia_staking::{Config, EraIndex, UseNominatorsMap};

frame_support::parameter_types! {
	pub const StakingPalletId: PalletId = PalletId(*b"da/staki");
	pub const SessionsPerEra: SessionIndex = DARWINIA_SESSIONS_PER_ERA;
	pub const BondingDurationInEra: EraIndex = BondingDurationInBlockNumber::get()
		/ (DARWINIA_SESSIONS_PER_ERA as BlockNumber * DARWINIA_BLOCKS_PER_SESSION);
	pub const BondingDurationInBlockNumber: BlockNumber = 14 * DAYS;
	// slightly less than 14 days.
	pub const SlashDeferDuration: EraIndex = BondingDurationInEra::get() - 1;
	// last 15 minutes of the last session will be for election.
	pub const MaxNominatorRewardedPerValidator: u32 = 64;
	pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(17);
	pub const Cap: Balance = RING_HARD_CAP;
	pub const TotalPower: Power = TOTAL_POWER;
}

impl Config for Runtime {
	type BondingDurationInBlockNumber = BondingDurationInBlockNumber;
	type BondingDurationInEra = BondingDurationInEra;
	type Cap = Cap;
	type ElectionProvider = ElectionProviderMultiPhase;
	type Event = Event;
	type GenesisElectionProvider = GenesisElectionOf<Self>;
	type KtonCurrency = Kton;
	type KtonReward = ();
	type KtonSlash = KtonTreasury;
	type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
	type NextNewSession = Session;
	type OffendingValidatorsThreshold = OffendingValidatorsThreshold;
	type PalletId = StakingPalletId;
	type RingCurrency = Ring;
	type RingReward = ();
	type RingRewardRemainder = Treasury;
	type RingSlash = Treasury;
	type SessionInterface = Self;
	type SessionsPerEra = SessionsPerEra;
	/// A super-majority of the council can cancel the slash.
	type SlashCancelOrigin = RootOrAtLeastHalf<CouncilCollective>;
	type SlashDeferDuration = SlashDeferDuration;
	// Use the nominator map to iter voter AND no-ops for all SortedListProvider hooks. The
	// migration to bags-list is a no-op, but the storage version will be updated.
	type SortedListProvider = UseNominatorsMap<Self>;
	type TotalPower = TotalPower;
	type UnixTime = Timestamp;
	type WeightInfo = ();

	const MAX_NOMINATIONS: u32 = <NposCompactSolution16 as NposSolution>::LIMIT as u32;
}
