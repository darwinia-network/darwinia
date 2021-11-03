// --- paritytech ---
use pallet_election_provider_multi_phase::Config;
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
	pub const UnsignedPhase: u32 = BLOCKS_PER_SESSION / 4;

	// fallback: run election on-chain.
	pub const Fallback: pallet_election_provider_multi_phase::FallbackStrategy =
		pallet_election_provider_multi_phase::FallbackStrategy::OnChain;

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
	type DataProvider = Staking;
	type OnChainAccuracy = Perbill;
	type CompactSolution = NposCompactSolution16;
	type Fallback = Fallback;
	type WeightInfo = WeightInfo<Runtime>;
	type ForceOrigin = EnsureRootOrHalfCouncil;
	type BenchmarkingConfig = ();
}
