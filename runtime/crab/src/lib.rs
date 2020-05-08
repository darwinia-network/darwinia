//! The Crab runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

/// Constant values used within the runtime.
pub mod constants;
use constants::{currency::*, fee::*, time::*};

// --- darwinia ---
#[cfg(feature = "std")]
pub use darwinia_claims::ClaimsList;
#[cfg(feature = "std")]
pub use darwinia_eth_relay::DagsMerkleRootsLoader;
#[cfg(feature = "std")]
pub use darwinia_staking::{Forcing, StakerStatus};

// --- crates ---
use codec::Encode;
// --- substrate ---
use frame_support::{
	construct_runtime, debug, parameter_types,
	traits::{Imbalance, LockIdentifier, OnUnbalanced, Randomness},
	weights::RuntimeDbWeight,
};
use pallet_grandpa::{fg_primitives, AuthorityList as GrandpaAuthorityList};
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
		BlakeTwo256, Block as BlockT, Extrinsic as ExtrinsicT, IdentityLookup, OpaqueKeys,
		SaturatedConversion,
	},
	transaction_validity::{TransactionPriority, TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, ModuleId, Perbill, Percent, Permill, Perquintill,
};
use sp_staking::SessionIndex;
use sp_std::prelude::*;
#[cfg(any(feature = "std", test))]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
// --- darwinia ---
use darwinia_balances_rpc_runtime_api::RuntimeDispatchInfo as BalancesRuntimeDispatchInfo;
use darwinia_eth_offchain::crypto::AuthorityId as EthOffchainId;
use darwinia_eth_relay::EthNetworkType;
use darwinia_primitives::*;
use darwinia_runtime_common::*;
use darwinia_staking::EraIndex;

type Ring = Balances;

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// Runtime version (Crab).
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("Crab"),
	impl_name: create_runtime_str!("Crab"),
	authoring_version: 0,
	spec_version: 0,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 0,
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
	pub const DbWeight: RuntimeDbWeight = RuntimeDbWeight {
		read: 60_000_000,
		write: 200_000_000,
	};
	pub const Version: RuntimeVersion = VERSION;
}
impl frame_system::Trait for Runtime {
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
	type DbWeight = DbWeight;
	type BlockExecutionWeight = BlockExecutionWeight;
	type ExtrinsicBaseWeight = ExtrinsicBaseWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = Version;
	type ModuleToIndex = ModuleToIndex;
	type AccountData = AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
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
}

parameter_types! {
	pub const MinimumPeriod: Moment = SLOT_DURATION / 2;
}
impl pallet_timestamp::Trait for Runtime {
	type Moment = Moment;
	type OnTimestampSet = Babe;
	type MinimumPeriod = MinimumPeriod;
}

parameter_types! {
	pub const IndexDeposit: Balance = 1 * COIN;
}
impl pallet_indices::Trait for Runtime {
	type AccountIndex = AccountIndex;
	type Currency = Ring;
	type Deposit = IndexDeposit;
	type Event = Event;
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
	pub const TransactionByteFee: Balance = 10 * MICRO;
	// for a sane configuration, this should always be less than `AvailableBlockRatio`.
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
}
impl pallet_transaction_payment::Trait for Runtime {
	type Currency = Ring;
	type OnTransactionPayment = DealWithFees;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = WeightToFee;
	type FeeMultiplierUpdate = TargetedFeeAdjustment<TargetBlockFullness, Self>;
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

impl pallet_offences::Trait for Runtime {
	type Event = Event;
	type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
	type OnOffenceHandler = Staking;
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
	type SessionManager = Staking;
	type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
}

parameter_types! {
	pub const WindowSize: BlockNumber = pallet_finality_tracker::DEFAULT_WINDOW_SIZE.into();
	pub const ReportLatency: BlockNumber = pallet_finality_tracker::DEFAULT_REPORT_LATENCY.into();
}
impl pallet_finality_tracker::Trait for Runtime {
	type OnFinalizationStalled = ();
	type WindowSize = WindowSize;
	type ReportLatency = ReportLatency;
}

impl pallet_grandpa::Trait for Runtime {
	type Event = Event;
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
}

impl pallet_authority_discovery::Trait for Runtime {}

// parameter_types! {
// 	pub const LaunchPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
// 	pub const VotingPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
// 	pub const EmergencyVotingPeriod: BlockNumber = 3 * 24 * 60 * MINUTES;
// 	pub const MinimumDeposit: Balance = 100 * COIN;
// 	pub const EnactmentPeriod: BlockNumber = 30 * 24 * 60 * MINUTES;
// 	pub const CooloffPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
// 	// One cent: $10,000 / MB
// 	pub const PreimageByteDeposit: Balance = 1 * MILLI;
// }
// impl pallet_democracy::Trait for Runtime {
// 	type Proposal = Call;
// 	type Event = Event;
// 	type Currency = Ring;
// 	type EnactmentPeriod = EnactmentPeriod;
// 	type LaunchPeriod = LaunchPeriod;
// 	type VotingPeriod = VotingPeriod;
// 	type MinimumDeposit = MinimumDeposit;
// 	/// A straight majority of the council can decide what their next motion is.
// 	type ExternalOrigin = pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>;
// 	/// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
// 	type ExternalMajorityOrigin = pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollective>;
// 	/// A unanimous council can have the next scheduled referendum be a straight default-carries
// 	/// (NTB) vote.
// 	type ExternalDefaultOrigin = pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>;
// 	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
// 	/// be tabled immediately and with a shorter voting/enactment period.
// 	type FastTrackOrigin = pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, TechnicalCollective>;
// 	type EmergencyVotingPeriod = EmergencyVotingPeriod;
// 	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
// 	type CancellationOrigin = pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollective>;
// 	// Any single technical committee member may veto a coming council proposal, however they can
// 	// only do it once and it lasts only for the cooloff period.
// 	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
// 	type CooloffPeriod = CooloffPeriod;
// 	type PreimageByteDeposit = PreimageByteDeposit;
// 	type Slash = Treasury;
// }

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 3 * DAYS;
	pub const TechnicalMotionDuration: BlockNumber = 3 * DAYS;
}
type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Trait<CouncilCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = CouncilMotionDuration;
}
type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Trait<TechnicalCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = TechnicalMotionDuration;
}

impl pallet_membership::Trait<pallet_membership::Instance1> for Runtime {
	type Event = Event;
	type AddOrigin =
		pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
	type RemoveOrigin =
		pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
	type SwapOrigin =
		pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
	type ResetOrigin =
		pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
	type PrimeOrigin =
		pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
	type MembershipInitialized = TechnicalCommittee;
	type MembershipChanged = TechnicalCommittee;
}

parameter_types! {
	// One storage item; value is size 4+4+16+32 bytes = 56 bytes.
	pub const MultisigDepositBase: Balance = 30 * MILLI;
	// Additional storage item size of 32 bytes.
	pub const MultisigDepositFactor: Balance = 5 * MILLI;
	pub const MaxSignatories: u16 = 100;
}
impl pallet_utility::Trait for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Ring;
	type MultisigDepositBase = MultisigDepositBase;
	type MultisigDepositFactor = MultisigDepositFactor;
	type MaxSignatories = MaxSignatories;
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
	type ForceOrigin =
		pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
	type RegistrarOrigin =
		pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
}

parameter_types! {
	pub const SocietyModuleId: ModuleId = ModuleId(*b"py/socie");
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
	type FounderSetOrigin =
		pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
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

impl pallet_sudo::Trait for Runtime {
	type Event = Event;
	type Call = Call;
}

parameter_types! {
	pub const RingExistentialDeposit: Balance = 100 * MILLI;
	pub const KtonExistentialDeposit: Balance = 10 * MICRO;
}
impl darwinia_balances::Trait<RingInstance> for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = RingExistentialDeposit;
	type BalanceInfo = AccountData<Balance>;
	type AccountStore = System;
	type DustCollector = (Kton,);
}
impl darwinia_balances::Trait<KtonInstance> for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = KtonExistentialDeposit;
	type BalanceInfo = AccountData<Balance>;
	type AccountStore = System;
	type DustCollector = (Ring,);
}

parameter_types! {
	pub const SessionsPerEra: SessionIndex = SESSIONS_PER_ERA;
	pub const BondingDurationInEra: EraIndex = 14 * DAYS
		/ (SESSIONS_PER_ERA as BlockNumber * BLOCKS_PER_SESSION);
	pub const BondingDurationInBlockNumber: BlockNumber = 14 * DAYS;
	pub const SlashDeferDuration: EraIndex = 14 * DAYS
		/ (SESSIONS_PER_ERA as BlockNumber * BLOCKS_PER_SESSION);
	pub const ElectionLookahead: BlockNumber = BLOCKS_PER_SESSION / 4;
	pub const MaxIterations: u32 = 5;
	pub const MaxNominatorRewardedPerValidator: u32 = 64;
	pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
	// quarter of the last session will be for election.
	pub const Cap: Balance = CAP;
	pub const TotalPower: Power = TOTAL_POWER;
}
impl darwinia_staking::Trait for Runtime {
	type Event = Event;
	type UnixTime = Timestamp;
	type SessionsPerEra = SessionsPerEra;
	type BondingDurationInEra = BondingDurationInEra;
	type BondingDurationInBlockNumber = BondingDurationInBlockNumber;
	type SlashDeferDuration = SlashDeferDuration;
	/// A super-majority of the council can cancel the slash.
	type SlashCancelOrigin =
		pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>;
	type SessionInterface = Self;
	type NextNewSession = Session;
	type ElectionLookahead = ElectionLookahead;
	type Call = Call;
	type MaxIterations = MaxIterations;
	type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
	type UnsignedPriority = StakingUnsignedPriority;
	type RingCurrency = Ring;
	type RingRewardRemainder = Treasury;
	type RingSlash = Treasury;
	type RingReward = ();
	type KtonCurrency = Kton;
	type KtonSlash = Treasury;
	type KtonReward = ();
	type Cap = Cap;
	type TotalPower = TotalPower;
}

parameter_types! {
	pub const ElectionsPhragmenModuleId: LockIdentifier = *b"phrelect";
	pub const CandidacyBond: Balance = 1 * COIN;
	pub const VotingBond: Balance = 5 * MILLI;
	/// Daily council elections.
	pub const TermDuration: BlockNumber = 24 * HOURS;
	pub const DesiredMembers: u32 = 13;
	pub const DesiredRunnersUp: u32 = 7;
}
impl darwinia_elections_phragmen::Trait for Runtime {
	type ModuleId = ElectionsPhragmenModuleId;
	type Event = Event;
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
}

parameter_types! {
	pub const TreasuryModuleId: ModuleId = ModuleId(*b"py/trsry");
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const RingProposalBondMinimum: Balance = 20 * COIN;
	pub const KtonProposalBondMinimum: Balance = 20 * COIN;
	pub const SpendPeriod: BlockNumber = 6 * DAYS;
	pub const Burn: Permill = Permill::from_percent(0);
	pub const TipCountdown: BlockNumber = 1 * DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 1 * COIN;
	pub const TipReportDepositPerByte: Balance = 1 * MILLI;
}
impl darwinia_treasury::Trait for Runtime {
	type ModuleId = TreasuryModuleId;
	type RingCurrency = Ring;
	type KtonCurrency = Kton;
	type ApproveOrigin =
		pallet_collective::EnsureProportionAtLeast<_3, _5, AccountId, CouncilCollective>;
	type RejectOrigin =
		pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
	type Tippers = ElectionsPhragmen;
	type TipCountdown = TipCountdown;
	type TipFindersFee = TipFindersFee;
	type TipReportDepositBase = TipReportDepositBase;
	type TipReportDepositPerByte = TipReportDepositPerByte;
	type Event = Event;
	type RingProposalRejection = Treasury;
	type KtonProposalRejection = Treasury;
	type ProposalBond = ProposalBond;
	type RingProposalBondMinimum = RingProposalBondMinimum;
	type KtonProposalBondMinimum = KtonProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
}

parameter_types! {
	pub const Prefix: &'static [u8] = b"Pay RINGs to the Crab account:";
}
impl darwinia_claims::Trait for Runtime {
	type Event = Event;
	type Prefix = Prefix;
	type RingCurrency = Ring;
}

parameter_types! {
	pub const EthBackingModuleId: ModuleId = ModuleId(*b"da/backi");
	pub const SubKeyPrefix: u8 = 42;
}
impl darwinia_eth_backing::Trait for Runtime {
	type ModuleId = EthBackingModuleId;
	type Event = Event;
	type DetermineAccountId = darwinia_eth_backing::AccountIdDeterminator<Runtime>;
	type EthRelay = EthRelay;
	type OnDepositRedeem = Staking;
	type RingCurrency = Ring;
	type KtonCurrency = Kton;
	type SubKeyPrefix = SubKeyPrefix;
}

parameter_types! {
	pub const EthRelayModuleId: ModuleId = ModuleId(*b"da/ethrl");
	pub const EthNetwork: EthNetworkType = EthNetworkType::Mainnet;
}
impl darwinia_eth_relay::Trait for Runtime {
	type ModuleId = EthRelayModuleId;
	type Event = Event;
	type EthNetwork = EthNetwork;
	type Call = Call;
	type Currency = Ring;
}

parameter_types! {
	pub const FetchInterval: BlockNumber = 3;
}
impl darwinia_eth_offchain::Trait for Runtime {
	type AuthorityId = EthOffchainId;
	type FetchInterval = FetchInterval;
}

impl darwinia_header_mmr::Trait for Runtime {}

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
			frame_system::CheckVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
			frame_system::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
			Default::default(),
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

construct_runtime!(
	pub enum Runtime
	where
		Block = Block,
		NodeBlock = darwinia_primitives::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		// --- substrate ---
		// Basic stuff; balances is uncallable initially.
		System: frame_system::{Module, Call, Storage, Config, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Storage},

		// Must be before session.
		Babe: pallet_babe::{Module, Call, Storage, Config, Inherent(Timestamp)},

		Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
		Indices: pallet_indices::{Module, Call, Storage, Config<T>, Event<T>},
		TransactionPayment: pallet_transaction_payment::{Module, Storage},

		// Consensus support.
		Authorship: pallet_authorship::{Module, Call, Storage},
		Offences: pallet_offences::{Module, Call, Storage, Event},
		Historical: pallet_session_historical::{Module},
		Session: pallet_session::{Module, Call, Storage, Config<T>, Event},
		FinalityTracker: pallet_finality_tracker::{Module, Call, Storage, Inherent},
		Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event},
		ImOnline: pallet_im_online::{Module, Call, Storage, Config<T>, Event<T>, ValidateUnsigned},
		AuthorityDiscovery: pallet_authority_discovery::{Module, Call, Config},

		// Governance stuff; uncallable initially.
		// Democracy: pallet_democracy::{Module, Call, Storage, Config, Event<T>},
		Council: pallet_collective::<Instance1>::{Module, Call, Storage, Origin<T>, Config<T>, Event<T>},
		TechnicalCommittee: pallet_collective::<Instance2>::{Module, Call, Storage, Origin<T>, Config<T>, Event<T>},
		TechnicalMembership: pallet_membership::<Instance1>::{Module, Call, Storage, Config<T>, Event<T>},

		// Utility module.
		Utility: pallet_utility::{Module, Call, Storage, Event<T>},

		// Less simple identity module.
		Identity: pallet_identity::{Module, Call, Storage, Event<T>},

		// Society module.
		Society: pallet_society::{Module, Call, Storage, Event<T>},

		// Social recovery module.
		Recovery: pallet_recovery::{Module, Call, Storage, Event<T>},

		Sudo: pallet_sudo::{Module, Call, Storage, Config<T>, Event<T>},

		// --- darwinia ---
		// Basic stuff; balances is uncallable initially.
		Balances: darwinia_balances::<Instance0>::{Module, Call, Storage, Config<T>, Event<T>},
		Kton: darwinia_balances::<Instance1>::{Module, Call, Storage, Config<T>, Event<T>},

		// Consensus support.
		Staking: darwinia_staking::{Module, Call, Storage, Config<T>, Event<T>, ValidateUnsigned},

		// Governance stuff; uncallable initially.
		ElectionsPhragmen: darwinia_elections_phragmen::{Module, Call, Storage, Config<T>, Event<T>},

		// Claims. Usable initially.
		Claims: darwinia_claims::{Module, Call, Storage, Config, Event<T>, ValidateUnsigned},

		EthBacking: darwinia_eth_backing::{Module, Call, Storage, Config<T>, Event<T>},
		EthRelay: darwinia_eth_relay::{Module, Call, Storage, Config<T>, Event<T>},
		EthOffchain: darwinia_eth_offchain::{Module, Call},

		HeaderMMR: darwinia_header_mmr::{Module, Call, Storage},

		// Governance stuff; uncallable initially.
		Treasury: darwinia_treasury::{Module, Call, Storage, Event<T>},
	}
);

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
	frame_system::CheckVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
	darwinia_eth_relay::CheckEthRelayHeaderHash<Runtime>,
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
>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;

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
		UncheckedExtrinsic,
	> for Runtime {
		fn query_info(
			uxt: UncheckedExtrinsic, len: u32
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

	// TODO: benchmark
	// #[cfg(feature = "runtime-benchmarks")]
	// impl frame_benchmarking::Benchmark<Block> for Runtime {
	// 	fn dispatch_benchmark(
	// 		module: Vec<u8>,
	// 		extrinsic: Vec<u8>,
	// 		lowest_range_values: Vec<u32>,
	// 		highest_range_values: Vec<u32>,
	// 		steps: Vec<u32>,
	// 		repeat: u32,
	// 	) -> Result<Vec<frame_benchmarking::BenchmarkResults>, RuntimeString> {
	// 		use frame_benchmarking::Benchmarking;

	// 		let result = match module.as_slice() {
	// 			b"claims" => Claims::run_benchmark(
	// 				extrinsic,
	// 				lowest_range_values,
	// 				highest_range_values,
	// 				steps,
	// 				repeat,
	// 			),
	// 			_ => Err("Benchmark not found for this pallet."),
	// 		};

	// 		result.map_err(|e| e.into())
	// 	}
	// }
}
