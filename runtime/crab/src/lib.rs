//! The Crab runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

/// Constant values used within the runtime.
pub mod constants;

pub mod wasm {
	//! Make the WASM binary available.

	#[cfg(all(feature = "std", any(target_arch = "x86_64", target_arch = "x86")))]
	include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

	#[cfg(all(feature = "std", not(any(target_arch = "x86_64", target_arch = "x86"))))]
	pub const WASM_BINARY: &[u8] = include_bytes!("../../../wasm/crab_runtime.compact.wasm");
	#[cfg(all(feature = "std", not(any(target_arch = "x86_64", target_arch = "x86"))))]
	pub const WASM_BINARY_BLOATY: &[u8] = include_bytes!("../../../wasm/crab_runtime.wasm");

	#[cfg(feature = "std")]
	/// Wasm binary unwrapped. If built with `BUILD_DUMMY_WASM_BINARY`, the function panics.
	pub fn wasm_binary_unwrap() -> &'static [u8] {
		#[cfg(all(feature = "std", any(target_arch = "x86_64", target_arch = "x86")))]
		return WASM_BINARY.expect(
			"Development wasm binary is not available. This means the client is \
						built with `BUILD_DUMMY_WASM_BINARY` flag and it is only usable for \
						production chains. Please rebuild with the flag disabled.",
		);
		#[cfg(all(feature = "std", not(any(target_arch = "x86_64", target_arch = "x86"))))]
		return WASM_BINARY;
	}
}

/// Weights for pallets used in the runtime.
mod weights;

// --- darwinia ---
#[cfg(feature = "std")]
pub use darwinia_claims::ClaimsList;
#[cfg(feature = "std")]
pub use darwinia_ethereum_relay::DagsMerkleRootsLoader;
#[cfg(feature = "std")]
pub use darwinia_staking::{Forcing, StakerStatus};
pub use wasm::*;

// --- crates ---
use codec::{Decode, Encode};
use static_assertions::const_assert;
// --- substrate ---
use frame_support::{
	construct_runtime, debug, parameter_types,
	traits::{
		ChangeMembers, Imbalance, InstanceFilter, KeyOwnerProofSystem, LockIdentifier,
		OnUnbalanced, Randomness,
	},
	weights::Weight,
};
use frame_system::{EnsureOneOf, EnsureRoot};
use pallet_grandpa::{
	fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_session::historical as pallet_session_historical;
use pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo as TransactionPaymentRuntimeDispatchInfo;
use sp_api::impl_runtime_apis;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_core::{
	u32_trait::{_1, _2, _3, _5},
	OpaqueMetadata,
};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{
		BlakeTwo256, Block as BlockT, Extrinsic as ExtrinsicT, IdentityLookup, NumberFor,
		OpaqueKeys, SaturatedConversion, Verify,
	},
	transaction_validity::{TransactionPriority, TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, KeyTypeId, ModuleId, Perbill, Percent, Permill, RuntimeDebug,
};
use sp_staking::SessionIndex;
use sp_std::prelude::*;
#[cfg(any(feature = "std", test))]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
// --- darwinia ---
use constants::{currency::*, fee::*, relay::*, time::*};
use darwinia_balances_rpc_runtime_api::RuntimeDispatchInfo as BalancesRuntimeDispatchInfo;
use darwinia_header_mmr_rpc_runtime_api::RuntimeDispatchInfo as HeaderMMRRuntimeDispatchInfo;
use darwinia_primitives::*;
use darwinia_relay_primitives::relay_authorities::OpCode;
use darwinia_runtime_common::*;
use darwinia_staking::EraIndex;
use darwinia_staking_rpc_runtime_api::RuntimeDispatchInfo as StakingRuntimeDispatchInfo;
use ethereum_primitives::EthereumNetworkType;

/// The address format for describing accounts.
pub type Address = AccountId;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
	darwinia_ethereum_relay::CheckEthereumRelayHeaderParcel<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Nonce, Call>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllModules,
	// CustomOnRuntimeUpgrade,
>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;

type Ring = Balances;

/// Runtime version (Crab).
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("Crab"),
	impl_name: create_runtime_str!("Darwinia Crab"),
	authoring_version: 0,
	spec_version: 37,
	impl_version: 0,
	#[cfg(not(feature = "disable-runtime-api"))]
	apis: RUNTIME_API_VERSIONS,
	#[cfg(feature = "disable-runtime-api")]
	apis: sp_version::create_apis_vec![[]],
	transaction_version: 3,
};

/// Native version.
#[cfg(any(feature = "std", test))]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
}
impl frame_system::Trait for Runtime {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Call = Call;
	type Index = Nonce;
	type BlockNumber = BlockNumber;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = RocksDbWeight;
	type BlockExecutionWeight = BlockExecutionWeight;
	type ExtrinsicBaseWeight = ExtrinsicBaseWeight;
	type MaximumExtrinsicWeight = MaximumExtrinsicWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = weights::frame_system::WeightInfo<Runtime>;
}

parameter_types! {
	pub const EpochDuration: u64 = BLOCKS_PER_SESSION as _;
	pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
}
impl pallet_babe::Trait for Runtime {
	type EpochDuration = EpochDuration;
	type ExpectedBlockTime = ExpectedBlockTime;
	// session module is the trigger
	type EpochChangeTrigger = pallet_babe::ExternalTrigger;
	type KeyOwnerProofSystem = Historical;
	type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		pallet_babe::AuthorityId,
	)>>::Proof;
	type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		pallet_babe::AuthorityId,
	)>>::IdentificationTuple;
	type HandleEquivocation =
		pallet_babe::EquivocationHandler<Self::KeyOwnerIdentification, Offences>;
	type WeightInfo = ();
}

parameter_types! {
	pub const MinimumPeriod: Moment = SLOT_DURATION / 2;
}
impl pallet_timestamp::Trait for Runtime {
	type Moment = Moment;
	type OnTimestampSet = Babe;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = weights::pallet_timestamp::WeightInfo<Runtime>;
}

parameter_types! {
	pub const IndexDeposit: Balance = 1 * COIN;
}
impl pallet_indices::Trait for Runtime {
	type AccountIndex = AccountIndex;
	type Currency = Ring;
	type Deposit = IndexDeposit;
	type Event = Event;
	type WeightInfo = weights::pallet_indices::WeightInfo<Runtime>;
}

parameter_types! {
	pub const RingExistentialDeposit: Balance = 100 * MILLI;
	pub const KtonExistentialDeposit: Balance = 10 * MICRO;
	pub const MaxLocks: u32 = 50;
}
impl darwinia_balances::Trait<RingInstance> for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = RingExistentialDeposit;
	type BalanceInfo = AccountData<Balance>;
	type AccountStore = System;
	type OtherCurrencies = (Kton,);
	type MaxLocks = MaxLocks;
	type WeightInfo = weights::darwinia_balances::WeightInfo<Runtime>;
}
impl darwinia_balances::Trait<KtonInstance> for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = KtonExistentialDeposit;
	type BalanceInfo = AccountData<Balance>;
	type AccountStore = System;
	type OtherCurrencies = (Ring,);
	type MaxLocks = MaxLocks;
	type WeightInfo = weights::darwinia_balances::WeightInfo<Runtime>;
}

pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance<Runtime>> for DealWithFees {
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance<Runtime>>) {
		if let Some(fees) = fees_then_tips.next() {
			// for fees, 80% to treasury, 20% to author
			let mut split = fees.ration(80, 20);
			if let Some(tips) = fees_then_tips.next() {
				// for tips, if any, 80% to treasury, 20% to author (though this can be anything)
				tips.ration_merge_into(80, 20, &mut split);
			}
			Treasury::on_unbalanced(split.0);
			ToAuthor::on_unbalanced(split.1);
		}
	}
}
parameter_types! {
	pub const TransactionByteFee: Balance = 5 * MILLI;
}
impl pallet_transaction_payment::Trait for Runtime {
	type Currency = Ring;
	type OnTransactionPayment = DealWithFees;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = WeightToFee;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
}

parameter_types! {
	pub const UncleGenerations: BlockNumber = 0;
}
// TODO: substrate#2986 implement this properly
impl pallet_authorship::Trait for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = (Staking, ImOnline);
}

parameter_types! {
	pub const StakingModuleId: ModuleId = ModuleId(*b"da/staki");
	pub const SessionsPerEra: SessionIndex = SESSIONS_PER_ERA;
	pub const BondingDurationInEra: EraIndex = 14 * DAYS
		/ (SESSIONS_PER_ERA as BlockNumber * BLOCKS_PER_SESSION);
	pub const BondingDurationInBlockNumber: BlockNumber = 14 * DAYS;
	// slightly less than 14 days.
	pub const SlashDeferDuration: EraIndex = 14 * DAYS
		/ (SESSIONS_PER_ERA as BlockNumber * BLOCKS_PER_SESSION) - 1;
	// quarter of the last session will be for election.
	pub const ElectionLookahead: BlockNumber = BLOCKS_PER_SESSION / 4;
	pub const MaxIterations: u32 = 5;
	pub MinSolutionScoreBump: Perbill = Perbill::from_rational_approximation(5u32, 10_000);
	pub const MaxNominatorRewardedPerValidator: u32 = 64;
	pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
	pub OffchainSolutionWeightLimit: Weight = MaximumExtrinsicWeight::get()
		.saturating_sub(BlockExecutionWeight::get())
		.saturating_sub(ExtrinsicBaseWeight::get());
	pub const Cap: Balance = CAP;
	pub const TotalPower: Power = TOTAL_POWER;
}
impl darwinia_staking::Trait for Runtime {
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
	type ElectionLookahead = ElectionLookahead;
	type Call = Call;
	type MaxIterations = MaxIterations;
	type MinSolutionScoreBump = MinSolutionScoreBump;
	type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
	type UnsignedPriority = StakingUnsignedPriority;
	// The unsigned solution weight targeted by the OCW. We set it to the maximum possible value of
	// a single extrinsic.
	type OffchainSolutionWeightLimit = OffchainSolutionWeightLimit;
	type RingCurrency = Ring;
	type RingRewardRemainder = Treasury;
	type RingSlash = Treasury;
	type RingReward = ();
	type KtonCurrency = Kton;
	type KtonSlash = Treasury;
	type KtonReward = ();
	type Cap = Cap;
	type TotalPower = TotalPower;
	type WeightInfo = weights::darwinia_staking::WeightInfo<Runtime>;
}

parameter_types! {
	pub OffencesWeightSoftLimit: Weight = Perbill::from_percent(60) * MaximumBlockWeight::get();
}
impl pallet_offences::Trait for Runtime {
	type Event = Event;
	type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
	type OnOffenceHandler = Staking;
	type WeightSoftLimit = OffencesWeightSoftLimit;
}

impl pallet_session::historical::Trait for Runtime {
	type FullIdentification = darwinia_staking::Exposure<AccountId, Balance, Balance>;
	type FullIdentificationOf = darwinia_staking::ExposureOf<Runtime>;
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub babe: Babe,
		pub grandpa: Grandpa,
		pub im_online: ImOnline,
		pub authority_discovery: AuthorityDiscovery,
	}
}
parameter_types! {
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}
impl pallet_session::Trait for Runtime {
	type Event = Event;
	type ValidatorId = AccountId;
	type ValidatorIdOf = darwinia_staking::StashOf<Self>;
	type ShouldEndSession = Babe;
	type NextSessionRotation = Babe;
	type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
	type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
	type WeightInfo = weights::pallet_session::WeightInfo<Runtime>;
}

parameter_types! {
	pub WindowSize: BlockNumber = pallet_finality_tracker::DEFAULT_WINDOW_SIZE.into();
	pub ReportLatency: BlockNumber = pallet_finality_tracker::DEFAULT_REPORT_LATENCY.into();
}
impl pallet_finality_tracker::Trait for Runtime {
	type OnFinalizationStalled = ();
	type WindowSize = WindowSize;
	type ReportLatency = ReportLatency;
}

impl pallet_grandpa::Trait for Runtime {
	type Event = Event;
	type Call = Call;
	type KeyOwnerProof =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
	type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		GrandpaId,
	)>>::IdentificationTuple;
	type KeyOwnerProofSystem = Historical;
	type HandleEquivocation =
		pallet_grandpa::EquivocationHandler<Self::KeyOwnerIdentification, Offences>;
	type WeightInfo = ();
}

parameter_types! {
	pub const SessionDuration: BlockNumber = BLOCKS_PER_SESSION as _;
	pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
}
impl pallet_im_online::Trait for Runtime {
	type AuthorityId = ImOnlineId;
	type Event = Event;
	type SessionDuration = SessionDuration;
	type ReportUnresponsiveness = Offences;
	type UnsignedPriority = ImOnlineUnsignedPriority;
	type WeightInfo = weights::pallet_im_online::WeightInfo<Runtime>;
}

impl pallet_authority_discovery::Trait for Runtime {}

type EnsureRootOrHalfCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>,
>;
parameter_types! {
	pub const LaunchPeriod: BlockNumber = 7 * DAYS;
	pub const VotingPeriod: BlockNumber = 7 * DAYS;
	pub const FastTrackVotingPeriod: BlockNumber = 3 * HOURS;
	pub const MinimumDeposit: Balance = 1 * MILLI;
	pub const EnactmentPeriod: BlockNumber = 8 * DAYS;
	pub const CooloffPeriod: BlockNumber = 7 * DAYS;
	pub const PreimageByteDeposit: Balance = 10 * NANO;
	pub const InstantAllowed: bool = true;
	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = 100;
}
impl darwinia_democracy::Trait for Runtime {
	type Proposal = Call;
	type Event = Event;
	type Currency = Ring;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type MinimumDeposit = MinimumDeposit;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin = EnsureRootOrHalfCouncil;
	/// A majority can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin = EnsureRootOrHalfCouncil;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin =
		pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin =
		pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, TechnicalCollective>;
	type InstantOrigin =
		pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>;
	type InstantAllowed = InstantAllowed;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin =
		pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollective>;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>,
	>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cooloff period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
	type CooloffPeriod = CooloffPeriod;
	type PreimageByteDeposit = PreimageByteDeposit;
	type Slash = Treasury;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = MaxVotes;
	type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
	type MaxProposals = MaxProposals;
	type WeightInfo = weights::darwinia_democracy::WeightInfo<Runtime>;
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 3 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
	pub const TechnicalMotionDuration: BlockNumber = 3 * DAYS;
	pub const TechnicalMaxProposals: u32 = 100;
	pub const TechnicalMaxMembers: u32 = 100;
}
type CouncilCollective = pallet_collective::Instance0;
impl pallet_collective::Trait<CouncilCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
}
type TechnicalCollective = pallet_collective::Instance1;
impl pallet_collective::Trait<TechnicalCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = TechnicalMaxProposals;
	type MaxMembers = TechnicalMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
}

parameter_types! {
	pub const ElectionsPhragmenModuleId: LockIdentifier = *b"phrelect";
	pub const CandidacyBond: Balance = 1 * COIN;
	pub const VotingBond: Balance = 5 * MILLI;
	/// Daily council elections.
	pub const TermDuration: BlockNumber = 24 * HOURS;
	pub const DesiredMembers: u32 = 7;
	pub const DesiredRunnersUp: u32 = 7;
}
// Make sure that there are no more than MaxMembers members elected via phragmen.
const_assert!(DesiredMembers::get() <= CouncilMaxMembers::get());
impl darwinia_elections_phragmen::Trait for Runtime {
	type Event = Event;
	type ModuleId = ElectionsPhragmenModuleId;
	type Currency = Ring;
	type ChangeMembers = Council;
	type InitializeMembers = Council;
	type CurrencyToVote = support_kton_in_the_future::CurrencyToVoteHandler<Self>;
	type CandidacyBond = CandidacyBond;
	type VotingBond = VotingBond;
	type LoserCandidate = Treasury;
	type BadReport = Treasury;
	type KickedMember = Treasury;
	type DesiredMembers = DesiredMembers;
	type DesiredRunnersUp = DesiredRunnersUp;
	type TermDuration = TermDuration;
	type WeightInfo = weights::darwinia_elections_phragmen::WeightInfo<Runtime>;
}

pub struct MembershipChangedGroup;
impl ChangeMembers<AccountId> for MembershipChangedGroup {
	fn change_members_sorted(
		incoming: &[AccountId],
		outgoing: &[AccountId],
		sorted_new: &[AccountId],
	) {
		TechnicalCommittee::change_members_sorted(incoming, outgoing, sorted_new);
		EthereumRelay::change_members_sorted(incoming, outgoing, sorted_new);
	}
}
type EnsureRootOrMoreThanHalfCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>,
>;
impl pallet_membership::Trait<pallet_membership::Instance0> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrMoreThanHalfCouncil;
	type RemoveOrigin = EnsureRootOrMoreThanHalfCouncil;
	type SwapOrigin = EnsureRootOrMoreThanHalfCouncil;
	type ResetOrigin = EnsureRootOrMoreThanHalfCouncil;
	type PrimeOrigin = EnsureRootOrMoreThanHalfCouncil;
	type MembershipInitialized = TechnicalCommittee;
	type MembershipChanged = MembershipChangedGroup;
}

type ApproveOrigin = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_3, _5, AccountId, CouncilCollective>,
>;
parameter_types! {
	pub const TreasuryModuleId: ModuleId = ModuleId(*b"da/trsry");
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const RingProposalBondMinimum: Balance = 20 * COIN;
	pub const KtonProposalBondMinimum: Balance = 20 * COIN;
	pub const SpendPeriod: BlockNumber = 6 * DAYS;
	pub const Burn: Permill = Permill::from_percent(0);
	pub const TipCountdown: BlockNumber = 1 * DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 1 * COIN;
	pub const DataDepositPerByte: Balance = 1 * MILLI;
	pub const BountyDepositBase: Balance = 1 * COIN;
	pub const BountyDepositPayoutDelay: BlockNumber = 4 * DAYS;
	pub const BountyUpdatePeriod: BlockNumber = 90 * DAYS;
	pub const MaximumReasonLength: u32 = 16384;
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: Balance = 2 * COIN;
}
impl darwinia_treasury::Trait for Runtime {
	type ModuleId = TreasuryModuleId;
	type RingCurrency = Ring;
	type KtonCurrency = Kton;
	type ApproveOrigin = ApproveOrigin;
	type RejectOrigin = EnsureRootOrMoreThanHalfCouncil;
	type Tippers = ElectionsPhragmen;
	type TipCountdown = TipCountdown;
	type TipFindersFee = TipFindersFee;
	type TipReportDepositBase = TipReportDepositBase;
	type DataDepositPerByte = DataDepositPerByte;
	type Event = Event;
	type OnSlashRing = Treasury;
	type OnSlashKton = Treasury;
	type ProposalBond = ProposalBond;
	type RingProposalBondMinimum = RingProposalBondMinimum;
	type KtonProposalBondMinimum = KtonProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type MaximumReasonLength = MaximumReasonLength;
	type BountyCuratorDeposit = BountyCuratorDeposit;
	type BountyValueMinimum = BountyValueMinimum;
	type RingBurnDestination = Society;
	type KtonBurnDestination = ();
	type WeightInfo = weights::darwinia_treasury::WeightInfo<Runtime>;
}

parameter_types! {
	pub const ClaimsModuleId: ModuleId = ModuleId(*b"da/claim");
	pub Prefix: &'static [u8] = b"Pay RINGs to the Crab account:";
}
impl darwinia_claims::Trait for Runtime {
	type Event = Event;
	type ModuleId = ClaimsModuleId;
	type Prefix = Prefix;
	type RingCurrency = Ring;
	type MoveClaimOrigin = EnsureRootOrMoreThanHalfCouncil;
}

impl pallet_utility::Trait for Runtime {
	type Event = Event;
	type Call = Call;
	type WeightInfo = weights::pallet_utility::WeightInfo<Runtime>;
}

parameter_types! {
	// Minimum 100 bytes/CRING deposited (1 MILLI/byte)
	pub const BasicDeposit: Balance = 10 * COIN;       // 258 bytes on-chain
	pub const FieldDeposit: Balance = 250 * MILLI;     // 66 bytes on-chain
	pub const SubAccountDeposit: Balance = 2 * COIN;   // 53 bytes on-chain
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}
impl pallet_identity::Trait for Runtime {
	type Event = Event;
	type Currency = Ring;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = Treasury;
	type ForceOrigin = EnsureRootOrMoreThanHalfCouncil;
	type RegistrarOrigin = EnsureRootOrMoreThanHalfCouncil;
	type WeightInfo = weights::pallet_identity::WeightInfo<Runtime>;
}

parameter_types! {
	pub const SocietyModuleId: ModuleId = ModuleId(*b"da/socie");
	pub const CandidateDeposit: Balance = 10 * COIN;
	pub const WrongSideDeduction: Balance = 2 * COIN;
	pub const MaxStrikes: u32 = 10;
	pub const RotationPeriod: BlockNumber = 80 * HOURS;
	pub const PeriodSpend: Balance = 500 * COIN;
	pub const MaxLockDuration: BlockNumber = 36 * 30 * DAYS;
	pub const ChallengePeriod: BlockNumber = 7 * DAYS;
}
impl pallet_society::Trait for Runtime {
	type Event = Event;
	type ModuleId = SocietyModuleId;
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
	type SuspensionJudgementOrigin = pallet_society::EnsureFounder<Runtime>;
	type ChallengePeriod = ChallengePeriod;
}

parameter_types! {
	pub const ConfigDepositBase: Balance = 5 * COIN;
	pub const FriendDepositFactor: Balance = 50 * MILLI;
	pub const MaxFriends: u16 = 9;
	pub const RecoveryDeposit: Balance = 5 * COIN;
}
impl pallet_recovery::Trait for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Ring;
	type ConfigDepositBase = ConfigDepositBase;
	type FriendDepositFactor = FriendDepositFactor;
	type MaxFriends = MaxFriends;
	type RecoveryDeposit = RecoveryDeposit;
}

parameter_types! {
	pub const MaxScheduledPerBlock: u32 = 50;
}
impl pallet_scheduler::Trait for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumBlockWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = weights::pallet_scheduler::WeightInfo<Runtime>;
}

/// The type used to represent the kinds of proxying allowed.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug)]
pub enum ProxyType {
	Any,
	NonTransfer,
	Governance,
	Staking,
	IdentityJudgement,
	EthereumBridge,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}
impl InstanceFilter<Call> for ProxyType {
	fn filter(&self, c: &Call) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => matches!(
				c,
				Call::System(..) |
				Call::Babe(..) |
				Call::Timestamp(..) |
				Call::Indices(pallet_indices::Call::claim(..)) |
				Call::Indices(pallet_indices::Call::free(..)) |
				Call::Indices(pallet_indices::Call::freeze(..)) |
				// Specifically omitting Indices `transfer`, `force_transfer`
				// Specifically omitting the entire Balances pallet
				Call::Authorship(..) |
				Call::Democracy(..) |
				Call::Staking(..) |
				Call::Offences(..) |
				Call::Session(..) |
				Call::FinalityTracker(..) |
				Call::Grandpa(..) |
				Call::ImOnline(..) |
				Call::AuthorityDiscovery(..) |
				Call::Council(..) |
				Call::TechnicalCommittee(..) |
				Call::ElectionsPhragmen(..) |
				Call::TechnicalMembership(..) |
				Call::Treasury(..) |
				Call::Claims(..) |
				Call::Utility(..) |
				Call::Identity(..) |
				Call::Society(..) |
				Call::Recovery(pallet_recovery::Call::as_recovered(..)) |
				Call::Recovery(pallet_recovery::Call::vouch_recovery(..)) |
				Call::Recovery(pallet_recovery::Call::claim_recovery(..)) |
				Call::Recovery(pallet_recovery::Call::close_recovery(..)) |
				Call::Recovery(pallet_recovery::Call::remove_recovery(..)) |
				Call::Recovery(pallet_recovery::Call::cancel_recovered(..)) |
				Call::Scheduler(..) |
				Call::Proxy(..) |
				Call::Multisig(..) |
				// Specifically omitting the entire EthereumBacking pallet
				Call::EthereumRelay(..) |
				Call::HeaderMMR(..) // Specifically omitting the entire CrabIssuing pallet
				                    // Specifically omitting the entire EthereumRelayAuthorities pallet
			),
			ProxyType::Governance => matches!(
				c,
				Call::Democracy(..)
					| Call::Council(..) | Call::TechnicalCommittee(..)
					| Call::ElectionsPhragmen(..)
					| Call::Treasury(..) | Call::Utility(..)
			),
			ProxyType::Staking => matches!(c, Call::Staking(..) | Call::Utility(..)),
			ProxyType::IdentityJudgement => matches!(
				c,
				Call::Identity(pallet_identity::Call::provide_judgement(..))
					| Call::Utility(pallet_utility::Call::batch(..))
			),
			ProxyType::EthereumBridge => matches!(
				c,
				Call::EthereumBacking(..)
					| Call::EthereumRelay(..)
					| Call::EthereumRelayAuthorities(..)
			),
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::NonTransfer, _) => true,
			_ => false,
		}
	}
}
parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = deposit(0, 33);
	pub const MaxProxies: u16 = 32;
	pub const AnnouncementDepositBase: Balance = deposit(1, 8);
	pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
	pub const MaxPending: u16 = 32;
}
impl pallet_proxy::Trait for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type MaxPending = MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
	type WeightInfo = weights::pallet_proxy::WeightInfo<Runtime>;
}

parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = deposit(0, 32);
	pub const MaxSignatories: u16 = 100;
}
impl pallet_multisig::Trait for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Ring;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = weights::pallet_multisig::WeightInfo<Runtime>;
}

impl pallet_sudo::Trait for Runtime {
	type Event = Event;
	type Call = Call;
}

parameter_types! {
	pub const EthereumBackingModuleId: ModuleId = ModuleId(*b"da/ethbk");
	pub const EthereumBackingFeeModuleId: ModuleId = ModuleId(*b"da/ethfe");
	pub const RingLockLimit: Balance = 10_000_000 * COIN;
	pub const KtonLockLimit: Balance = 1_000 * COIN;
	// https://github.com/darwinia-network/darwinia-common/pull/377#issuecomment-730369387
	pub const AdvancedFee: Balance = 50 * COIN;
	pub const SyncReward: Balance = 1_000 * COIN;
}
impl darwinia_ethereum_backing::Trait for Runtime {
	type ModuleId = EthereumBackingModuleId;
	type FeeModuleId = EthereumBackingFeeModuleId;
	type Event = Event;
	type RedeemAccountId = AccountId;
	type EthereumRelay = EthereumRelay;
	type OnDepositRedeem = Staking;
	type RingCurrency = Ring;
	type KtonCurrency = Kton;
	type RingLockLimit = RingLockLimit;
	type KtonLockLimit = KtonLockLimit;
	type AdvancedFee = AdvancedFee;
	type SyncReward = SyncReward;
	type EcdsaAuthorities = EthereumRelayAuthorities;
	type WeightInfo = ();
}

type EthereumRelayAuthoritiesInstance = darwinia_relay_authorities::Instance0;
parameter_types! {
	pub const EthereumRelayAuthoritiesLockId: LockIdentifier = *b"ethrauth";
	pub const EthereumRelayAuthoritiesTermDuration: BlockNumber = 30 * DAYS;
	pub const MaxCandidates: usize = 7;
	pub const OpCodes: (OpCode, OpCode) = (
		[71, 159, 189, 249],
		[180, 188, 244, 151]
	);
	pub const SignThreshold: Perbill = Perbill::from_percent(60);
	pub const SubmitDuration: BlockNumber = 100;
}
impl darwinia_relay_authorities::Trait<EthereumRelayAuthoritiesInstance> for Runtime {
	type Event = Event;
	type RingCurrency = Ring;
	type LockId = EthereumRelayAuthoritiesLockId;
	type TermDuration = EthereumRelayAuthoritiesTermDuration;
	type MaxCandidates = MaxCandidates;
	type AddOrigin = ApproveOrigin;
	type RemoveOrigin = ApproveOrigin;
	type ResetOrigin = ApproveOrigin;
	type DarwiniaMMR = HeaderMMR;
	type Sign = EthereumBacking;
	type OpCodes = OpCodes;
	type SignThreshold = SignThreshold;
	type SubmitDuration = SubmitDuration;
	type WeightInfo = ();
}

type TechnicalCommitteeApproveOrigin = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_3, _5, AccountId, TechnicalCollective>,
>;
type EnsureRootOrHalfTechnicalComittee = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, TechnicalCollective>,
>;
parameter_types! {
	pub const EthereumRelayModuleId: ModuleId = ModuleId(*b"da/ethrl");
	pub const EthereumNetwork: EthereumNetworkType = EthereumNetworkType::Ropsten;
	pub const ConfirmPeriod: BlockNumber = 200;
	pub const ApproveThreshold: Perbill = Perbill::from_percent(60);
	pub const RejectThreshold: Perbill = Perbill::from_percent(1);
}
impl darwinia_ethereum_relay::Trait for Runtime {
	type ModuleId = EthereumRelayModuleId;
	type Event = Event;
	type EthereumNetwork = EthereumNetwork;
	type Call = Call;
	type Currency = Ring;
	type RelayerGame = EthereumRelayerGame;
	type ApproveOrigin = TechnicalCommitteeApproveOrigin;
	type RejectOrigin = EnsureRootOrHalfTechnicalComittee;
	type ConfirmPeriod = ConfirmPeriod;
	type TechnicalMembership = TechnicalMembership;
	type ApproveThreshold = ApproveThreshold;
	type RejectThreshold = RejectThreshold;
	type WeightInfo = ();
}

type EthereumRelayerGameInstance = darwinia_relayer_game::Instance0;
parameter_types! {
	pub const EthereumRelayerGameLockId: LockIdentifier = *b"da/rgame";
}
impl darwinia_relayer_game::Trait<EthereumRelayerGameInstance> for Runtime {
	type RingCurrency = Ring;
	type LockId = EthereumRelayerGameLockId;
	type RingSlash = Treasury;
	type RelayerGameAdjustor = EthereumRelayerGameAdjustor;
	type RelayableChain = EthereumRelay;
	type WeightInfo = ();
}

impl darwinia_header_mmr::Trait for Runtime {}

parameter_types! {
	pub const CrabIssuingModuleId: ModuleId = ModuleId(*b"da/crais");
}
impl darwinia_crab_issuing::Trait for Runtime {
	type Event = Event;
	type ModuleId = CrabIssuingModuleId;
	type RingCurrency = Ring;
	type WeightInfo = ();
}

construct_runtime!(
	pub enum Runtime
	where
		Block = Block,
		NodeBlock = OpaqueBlock,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		// Basic stuff; balances is uncallable initially.
		System: frame_system::{Module, Call, Storage, Config, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Storage},

		// Must be before session.
		Babe: pallet_babe::{Module, Call, Storage, Config, Inherent, ValidateUnsigned},

		Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
		Indices: pallet_indices::{Module, Call, Storage, Config<T>, Event<T>},
		TransactionPayment: pallet_transaction_payment::{Module, Storage},

		// Consensus support.
		Authorship: pallet_authorship::{Module, Call, Storage},
		Offences: pallet_offences::{Module, Call, Storage, Event},
		Historical: pallet_session_historical::{Module},
		Session: pallet_session::{Module, Call, Storage, Config<T>, Event},
		FinalityTracker: pallet_finality_tracker::{Module, Call, Storage, Inherent},
		Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event, ValidateUnsigned},
		ImOnline: pallet_im_online::{Module, Call, Storage, Config<T>, Event<T>, ValidateUnsigned},
		AuthorityDiscovery: pallet_authority_discovery::{Module, Call, Config},

		// Governance stuff; uncallable initially.
		Council: pallet_collective::<Instance0>::{Module, Call, Storage, Origin<T>, Config<T>, Event<T>},
		TechnicalCommittee: pallet_collective::<Instance1>::{Module, Call, Storage, Origin<T>, Config<T>, Event<T>},
		TechnicalMembership: pallet_membership::<Instance0>::{Module, Call, Storage, Config<T>, Event<T>},

		// Utility module.
		Utility: pallet_utility::{Module, Call, Event},

		// Less simple identity module.
		Identity: pallet_identity::{Module, Call, Storage, Event<T>},

		// Society module.
		Society: pallet_society::{Module, Call, Storage, Event<T>},

		// Social recovery module.
		Recovery: pallet_recovery::{Module, Call, Storage, Event<T>},

		// System scheduler.
		Scheduler: pallet_scheduler::{Module, Call, Storage, Event<T>},

		Sudo: pallet_sudo::{Module, Call, Storage, Config<T>, Event<T>},

		// Basic stuff; balances is uncallable initially.
		Balances: darwinia_balances::<Instance0>::{Module, Call, Storage, Config<T>, Event<T>},
		Kton: darwinia_balances::<Instance1>::{Module, Call, Storage, Config<T>, Event<T>},

		// Consensus support.
		Staking: darwinia_staking::{Module, Call, Storage, Config<T>, Event<T>, ValidateUnsigned},

		// Governance stuff; uncallable initially.
		ElectionsPhragmen: darwinia_elections_phragmen::{Module, Call, Storage, Config<T>, Event<T>},

		// Claims. Usable initially.
		Claims: darwinia_claims::{Module, Call, Storage, Config, Event<T>, ValidateUnsigned},

		EthereumBacking: darwinia_ethereum_backing::{Module, Call, Storage, Config<T>, Event<T>},
		EthereumRelay: darwinia_ethereum_relay::{Module, Call, Storage, Config<T>, Event<T>},
		EthereumRelayerGame: darwinia_relayer_game::<Instance0>::{Module, Storage},

		// Consensus support.
		HeaderMMR: darwinia_header_mmr::{Module, Call, Storage},

		// Governance stuff; uncallable initially.
		Treasury: darwinia_treasury::{Module, Call, Storage, Event<T>},

		// Proxy module. Late addition.
		Proxy: pallet_proxy::{Module, Call, Storage, Event<T>},

		// Multisig module. Late addition.
		Multisig: pallet_multisig::{Module, Call, Storage, Event<T>},

		// Crab bridge.
		CrabIssuing: darwinia_crab_issuing::{Module, Call, Storage, Config, Event<T>},

		// Governance stuff; uncallable initially.
		Democracy: darwinia_democracy::{Module, Call, Storage, Config, Event<T>},

		// Ethereum bridge.
		EthereumRelayAuthorities: darwinia_relay_authorities::<Instance0>::{Module, Call, Storage, Event<T>},
	}
);

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	Call: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: Call,
		public: <Signature as Verify>::Signer,
		account: AccountId,
		nonce: <Runtime as frame_system::Trait>::Index,
	) -> Option<(Call, <UncheckedExtrinsic as ExtrinsicT>::SignaturePayload)> {
		let period = BlockHashCount::get()
			.checked_next_power_of_two()
			.map(|c| c / 2)
			.unwrap_or(2) as u64;

		let current_block = System::block_number()
			.saturated_into::<u64>()
			.saturating_sub(1);
		let tip = 0;
		let extra: SignedExtra = (
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
			frame_system::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
			darwinia_ethereum_relay::CheckEthereumRelayHeaderParcel::<Runtime>::new(),
		);
		let raw_payload = SignedPayload::new(call, extra)
			.map_err(|e| {
				debug::warn!("Unable to create signed payload: {:?}", e);
			})
			.ok()?;
		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let (call, extra, _) = raw_payload.deconstruct();
		Some((call, (account, signature, extra)))
	}
}
impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature;
}
impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	Call: From<C>,
{
	type Extrinsic = UncheckedExtrinsic;
	type OverarchingCall = Call;
}

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			Runtime::metadata().into()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(
			data: sp_inherents::InherentData
		) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}

		fn random_seed() -> <Block as BlockT>::Hash {
			RandomnessCollectiveFlip::random_seed()
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
			Executive::validate_transaction(source, tx)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Grandpa::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
			)
		}

		fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
			authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			Historical::prove((fg_primitives::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(fg_primitives::OpaqueKeyOwnershipProof::new)
		}
	}

	impl sp_consensus_babe::BabeApi<Block> for Runtime {
		fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
			// The choice of `c` parameter (where `1 - c` represents the
			// probability of a slot being empty), is done in accordance to the
			// slot duration and expected target block time, for safely
			// resisting network delays of maximum two seconds.
			// <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
			sp_consensus_babe::BabeGenesisConfiguration {
				slot_duration: Babe::slot_duration(),
				epoch_length: EpochDuration::get(),
				c: PRIMARY_PROBABILITY,
				genesis_authorities: Babe::authorities(),
				randomness: Babe::randomness(),
				allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
			}
		}

		fn current_epoch_start() -> sp_consensus_babe::SlotNumber {
			Babe::current_epoch_start()
		}

		fn generate_key_ownership_proof(
			_slot_number: sp_consensus_babe::SlotNumber,
			authority_id: sp_consensus_babe::AuthorityId,
		) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
			Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
			key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Babe::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
			)
		}
	}
	impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
		fn authorities() -> Vec<AuthorityDiscoveryId> {
			AuthorityDiscovery::authorities()
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
		Block,
		Balance,
	> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic, len: u32
		) -> TransactionPaymentRuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
	}

	impl darwinia_balances_rpc_runtime_api::BalancesApi<Block, AccountId, Balance> for Runtime {
		fn usable_balance(
			instance: u8,
			account: AccountId
		) -> BalancesRuntimeDispatchInfo<Balance> {
			match instance {
				0 => Ring::usable_balance_rpc(account),
				1 => Kton::usable_balance_rpc(account),
				_ => Default::default()
			}
		}
	}

	impl darwinia_header_mmr_rpc_runtime_api::HeaderMMRApi<Block, Hash> for Runtime {
		fn gen_proof(
			block_number_of_member_leaf: u64,
			block_number_of_last_leaf: u64
		) -> HeaderMMRRuntimeDispatchInfo<Hash> {
			HeaderMMR::gen_proof_rpc(block_number_of_member_leaf, block_number_of_last_leaf )
		}
	}

	impl darwinia_staking_rpc_runtime_api::StakingApi<Block, AccountId, Power> for Runtime {
		fn power_of(account: AccountId) -> StakingRuntimeDispatchInfo<Power> {
			Staking::power_of_rpc(account)
		}
	}
}

// pub struct CustomOnRuntimeUpgrade;
// impl frame_support::traits::OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
// 	fn on_runtime_upgrade() -> frame_support::weights::Weight {
// 		// --- substrate ---
// 		use frame_support::migration::*;

// 		<Runtime as frame_system::Trait>::MaximumBlockWeight::get()
// 	}
// }
