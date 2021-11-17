// --- paritytech ---
use pallet_election_provider_multi_phase::{Config, FallbackStrategy};
use sp_runtime::{transaction_validity::TransactionPriority, Perbill};
// --- darwinia-network ---
use crate::{weights::pallet_election_provider_multi_phase::WeightInfo, *};

sp_npos_elections::generate_solution_type!(
	#[compact]
	pub struct NposCompactSolution24::<
		VoterIndex = u32,
		TargetIndex = u16,
		Accuracy = sp_runtime::PerU16,
	>(24)
);

frame_support::parameter_types! {
	// no signed phase for now, just unsigned.
	pub const SignedPhase: u32 = 0;
	pub const UnsignedPhase: u32 = BLOCKS_PER_SESSION / 4;

	// signed config
	pub const SignedMaxSubmissions: u32 = 10;
	pub const SignedRewardBase: Balance = 1 * MILLI;
	pub const SignedDepositBase: Balance = 1 * MILLI;
	pub const SignedDepositByte: Balance = 1 * MICRO;

	// fallback: no on-chain fallback.
	pub const Fallback: FallbackStrategy = FallbackStrategy::Nothing;

	pub SolutionImprovementThreshold: Perbill = Perbill::from_rational(5u32, 10_000);

	// miner configs
	pub NposSolutionPriority: TransactionPriority = Perbill::from_percent(90) * TransactionPriority::max_value();
	pub const MinerMaxIterations: u32 = 10;
	pub const OffchainRepeat: BlockNumber = 5;
}

impl Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type SignedPhase = SignedPhase;
	type UnsignedPhase = UnsignedPhase;
	type SolutionImprovementThreshold = SolutionImprovementThreshold;
	type MinerMaxIterations = MinerMaxIterations;
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
	type OnChainAccuracy = Perbill;
	type CompactSolution = NposCompactSolution24;
	type Fallback = Fallback;
	type WeightInfo = WeightInfo<Runtime>;
	type ForceOrigin = EnsureRootOrHalfCouncil;
	type BenchmarkingConfig = ();
}
