// --- substrate ---
use pallet_election_provider_multi_phase::Config;
use sp_runtime::{transaction_validity::TransactionPriority, Perbill};
// --- darwinia ---
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

	// fallback: run election on-chain.
	pub const Fallback: pallet_election_provider_multi_phase::FallbackStrategy =
		pallet_election_provider_multi_phase::FallbackStrategy::OnChain;

	pub SolutionImprovementThreshold: Perbill = Perbill::from_rational(5u32, 10_000);

	// miner configs
	pub NposSolutionPriority: TransactionPriority = Perbill::from_percent(90) * TransactionPriority::max_value();
	pub const MinerMaxIterations: u32 = 10;
}

impl Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type SignedPhase = SignedPhase;
	type UnsignedPhase = UnsignedPhase;
	type SolutionImprovementThreshold = SolutionImprovementThreshold;
	type MinerMaxIterations = MinerMaxIterations;
	type MinerMaxWeight = OffchainSolutionWeightLimit;
	type MinerMaxLength = OffchainSolutionLengthLimit;
	type MinerTxPriority = NposSolutionPriority;
	type DataProvider = Staking;
	type OnChainAccuracy = Perbill;
	type CompactSolution = NposCompactSolution24;
	type Fallback = Fallback;
	type WeightInfo = WeightInfo<Runtime>;
	type BenchmarkingConfig = ();
}
