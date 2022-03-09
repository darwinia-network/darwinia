// --- paritytech ---
use frame_election_provider_support::onchain::OnChainSequentialPhragmen;
use frame_support::PalletId;
use sp_npos_elections::NposSolution;
use sp_staking::SessionIndex;
// --- darwinia-network ---
use crate::*;
use darwinia_staking::{Config, EraIndex, UseNominatorsMap};

#[cfg(feature = "dev")]
frame_support::parameter_types! {
	pub const BondingDurationInEra: BlockNumber = 2;
	pub const BondingDurationInBlockNumber: BlockNumber = 2 * DARWINIA_SESSIONS_PER_ERA * DARWINIA_BLOCKS_PER_SESSION;
	pub const SlashDeferDuration: EraIndex = 1;
}
#[cfg(not(feature = "dev"))]
frame_support::parameter_types! {
	pub const BondingDurationInEra: EraIndex = BondingDurationInBlockNumber::get()
		/ (DARWINIA_SESSIONS_PER_ERA as BlockNumber * DARWINIA_BLOCKS_PER_SESSION);
	pub const BondingDurationInBlockNumber: BlockNumber = 14 * DAYS;
	// slightly less than 14 days.
	pub const SlashDeferDuration: EraIndex = BondingDurationInEra::get() - 1;
}
frame_support::parameter_types! {
	pub const StakingPalletId: PalletId = PalletId(*b"da/staki");
	pub const SessionsPerEra: SessionIndex = DARWINIA_SESSIONS_PER_ERA;
	// last 15 minutes of the last session will be for election.
	pub const MaxNominatorRewardedPerValidator: u32 = 64;
	pub const Cap: Balance = RING_HARD_CAP;
	pub const TotalPower: Power = TOTAL_POWER;
}

impl Config for Runtime {
	const MAX_NOMINATIONS: u32 = <NposCompactSolution16 as NposSolution>::LIMIT as u32;
	type Event = Event;
	type PalletId = StakingPalletId;
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
	type GenesisElectionProvider = GenesisElectionOf<Self>;
	// Use the nominator map to iter voter AND no-ops for all SortedListProvider hooks. The migration
	// to bags-list is a no-op, but the storage version will be updated.
	type SortedListProvider = UseNominatorsMap<Self>;
	type RingCurrency = Ring;
	type RingRewardRemainder = Treasury;
	type RingSlash = Treasury;
	type RingReward = ();
	type KtonCurrency = Kton;
	type KtonSlash = KtonTreasury;
	type KtonReward = ();
	type Cap = Cap;
	type TotalPower = TotalPower;
	type WeightInfo = ();
}
