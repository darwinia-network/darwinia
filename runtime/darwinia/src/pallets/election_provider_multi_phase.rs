// --- paritytech ---
use frame_election_provider_support::{onchain, SequentialPhragmen};
use pallet_election_provider_multi_phase::{
	BenchmarkingConfig, Config, FallbackStrategy, NoFallback, SolutionAccuracyOf,
};
use sp_runtime::{transaction_validity::TransactionPriority, PerU16, Perbill};
// --- darwinia-network ---
use crate::*;
use weights::pallet_election_provider_multi_phase::WeightInfo;

sp_npos_elections::generate_solution_type!(
	#[compact]
	pub struct NposCompactSolution16::<
		VoterIndex = u32,
		TargetIndex = u16,
		Accuracy = PerU16,
	>(16)
);

frame_support::parameter_types! {
	// no signed phase for now, just unsigned.
	pub const SignedPhase: u32 = 0;
	pub const UnsignedPhase: u32 = DARWINIA_BLOCKS_PER_SESSION / 4;

	// signed config
	pub const SignedMaxSubmissions: u32 = 10;
	pub const SignedRewardBase: Balance = 1 * MILLI;
	pub const SignedDepositBase: Balance = 1 * MILLI;
	pub const SignedDepositByte: Balance = 1 * MICRO;

	pub SolutionImprovementThreshold: Perbill = Perbill::from_rational(5u32, 10_000);

	// miner configs
	pub NposSolutionPriority: TransactionPriority = Perbill::from_percent(90) * TransactionPriority::max_value();
	pub OffchainRepeat: BlockNumber = 5;

	/// Whilst `UseNominatorsAndUpdateBagsList` or `UseNominatorsMap` is in use, this can still be a
	/// very large value. Once the `BagsList` is in full motion, staking might open its door to many
	/// more nominators, and this value should instead be what is a "safe" number (e.g. 22500).
	pub const VoterSnapshotPerBlock: u32 = 22_500;
}

impl Config for Runtime {
	type Event = Event;
	type Currency = Ring;
	type EstimateCallFee = TransactionPayment;
	type SignedPhase = SignedPhase;
	type UnsignedPhase = UnsignedPhase;
	type SolutionImprovementThreshold = SolutionImprovementThreshold;
	type MinerMaxWeight = OffchainSolutionWeightLimit;
	type MinerMaxLength = OffchainSolutionLengthLimit; // For now use the one from staking.
	type OffchainRepeat = OffchainRepeat;
	type MinerTxPriority = NposSolutionPriority;
	type SignedMaxSubmissions = SignedMaxSubmissions;
	type SignedRewardBase = SignedRewardBase;
	type SignedDepositBase = SignedDepositBase;
	type SignedDepositByte = SignedDepositByte;
	type SignedDepositWeight = ();
	type SignedMaxWeight = Self::MinerMaxWeight;
	type SlashHandler = (); // burn slashes
	type RewardHandler = (); // nothing to do upon rewards
	type DataProvider = Staking;
	type Solution = NposCompactSolution16;
	type Fallback = NoFallback<Self>;
	type Solver = SequentialPhragmen<AccountId, SolutionAccuracyOf<Self>, OffchainRandomBalancing>;
	type WeightInfo = WeightInfo<Runtime>;
	type ForceOrigin = EnsureRootOrHalfCouncil;
	type BenchmarkingConfig = ();
	type VoterSnapshotPerBlock = VoterSnapshotPerBlock;
}

impl onchain::Config for Runtime {
	type Accuracy = Perbill;
	type DataProvider = Staking;
}
