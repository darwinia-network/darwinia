// This file is part of Darwinia.
//
// Copyright (C) 2018-2021 Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

//! The Darwinia runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

/// Constant values used within the runtime.
pub mod constants;
pub use constants::{currency::*, fee::*, relay::*, time::*};

pub mod pallets;
pub use pallets::*;

#[cfg(feature = "std")]
pub mod genesis_loader {
	// --- std ---
	use std::fs::File;
	// --- crates ---
	use serde::{de::Error, Deserialize, Deserializer};
	// --- darwinia ---
	use crate::*;

	#[derive(Deserialize)]
	struct Account {
		target: String,
		#[serde(deserialize_with = "string_to_balance")]
		amount: Balance,
	}

	fn string_to_balance<'de, D>(deserializer: D) -> Result<Balance, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s: String = Deserialize::deserialize(deserializer)?;

		s.parse::<Balance>().map_err(Error::custom)
	}

	pub fn load_genesis_swap_from_file(path: &str) -> Result<Vec<(String, Balance)>, String> {
		serde_json::from_reader(File::open(path).map_err(|e| e.to_string())?)
			.map(|accounts: Vec<Account>| {
				accounts
					.into_iter()
					.map(|Account { target, amount }| (target, amount))
					.collect()
			})
			.map_err(|e| e.to_string())
	}
}

pub mod wasm {
	//! Make the WASM binary available.

	#[cfg(all(feature = "std", any(target_arch = "x86_64", target_arch = "x86")))]
	include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

	#[cfg(all(feature = "std", not(any(target_arch = "x86_64", target_arch = "x86"))))]
	pub const WASM_BINARY: &[u8] = include_bytes!("../../../wasm/darwinia_runtime.compact.wasm");
	#[cfg(all(feature = "std", not(any(target_arch = "x86_64", target_arch = "x86"))))]
	pub const WASM_BINARY_BLOATY: &[u8] = include_bytes!("../../../wasm/darwinia_runtime.wasm");

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
pub use darwinia_ethereum_relay::DagsMerkleRootsLoader;
#[cfg(feature = "std")]
pub use darwinia_staking::{Forcing, StakerStatus};
pub use wasm::*;

// --- crates ---
use codec::{Decode, Encode};
// --- substrate ---
use frame_support::{
	construct_runtime, debug,
	traits::{InstanceFilter, KeyOwnerProofSystem, LockIdentifier, Randomness},
	weights::Weight,
};
use frame_system::{EnsureOneOf, EnsureRoot};
use pallet_grandpa::{
	fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
use pallet_session::historical as pallet_session_historical;
use pallet_transaction_payment::FeeDetails;
use pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo as TransactionPaymentRuntimeDispatchInfo;
use sp_api::impl_runtime_apis;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_core::{
	u32_trait::{_1, _2, _3, _5},
	OpaqueMetadata,
};
use sp_runtime::{
	create_runtime_str, generic,
	traits::{
		AccountIdLookup, BlakeTwo256, Block as BlockT, ConvertInto, Extrinsic as ExtrinsicT,
		NumberFor, SaturatedConversion, StaticLookup, Verify,
	},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, ModuleId, MultiAddress, Perbill, RuntimeDebug,
};
use sp_std::prelude::*;
#[cfg(any(feature = "std", test))]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
// --- darwinia ---
use darwinia_balances_rpc_runtime_api::RuntimeDispatchInfo as BalancesRuntimeDispatchInfo;
use darwinia_header_mmr_rpc_runtime_api::RuntimeDispatchInfo as HeaderMMRRuntimeDispatchInfo;
use darwinia_primitives::*;
use darwinia_relay_primitives::relay_authorities::OpCode;
use darwinia_runtime_common::*;
use darwinia_staking_rpc_runtime_api::RuntimeDispatchInfo as StakingRuntimeDispatchInfo;
use ethereum_primitives::EthereumNetworkType;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;
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
	CustomOnRuntimeUpgrade,
>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;

type Ring = Balances;

/// Runtime version (Darwinia).
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("Darwinia"),
	impl_name: create_runtime_str!("Darwinia"),
	authoring_version: 0,
	spec_version: 23,
	impl_version: 0,
	#[cfg(not(feature = "disable-runtime-api"))]
	apis: RUNTIME_API_VERSIONS,
	#[cfg(feature = "disable-runtime-api")]
	apis: sp_version::create_apis_vec![[]],
	transaction_version: 2,
};

/// Native version.
#[cfg(any(feature = "std", test))]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

frame_support::parameter_types! {
	pub const MinVestedTransfer: Balance = 100 * MILLI;
}
impl darwinia_vesting::Config for Runtime {
	type Event = Event;
	type Currency = Ring;
	type BlockNumberToBalance = ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = weights::darwinia_vesting::WeightInfo<Runtime>;
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type WeightInfo = weights::pallet_utility::WeightInfo<Runtime>;
}

frame_support::parameter_types! {
	// Minimum 100 bytes/CRING deposited (1 MILLI/byte)
	pub const BasicDeposit: Balance = deposit(1, 258);
	pub const FieldDeposit: Balance = deposit(0, 66);
	pub const SubAccountDeposit: Balance = deposit(1, 53);
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}
impl pallet_identity::Config for Runtime {
	type Event = Event;
	type Currency = Ring;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = Treasury;
	type ForceOrigin = EnsureRootOrHalfCouncil;
	type RegistrarOrigin = EnsureRootOrHalfCouncil;
	type WeightInfo = weights::pallet_identity::WeightInfo<Runtime>;
}

frame_support::parameter_types! {
	pub const SocietyModuleId: ModuleId = ModuleId(*b"da/socie");
	pub const CandidateDeposit: Balance = 10 * COIN;
	pub const WrongSideDeduction: Balance = 2 * COIN;
	pub const MaxStrikes: u32 = 10;
	pub const RotationPeriod: BlockNumber = 80 * HOURS;
	pub const PeriodSpend: Balance = 500 * COIN;
	pub const MaxLockDuration: BlockNumber = 36 * 30 * DAYS;
	pub const ChallengePeriod: BlockNumber = 7 * DAYS;
}
impl pallet_society::Config for Runtime {
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
	type FounderSetOrigin = EnsureRootOrHalfCouncil;
	type SuspensionJudgementOrigin = pallet_society::EnsureFounder<Runtime>;
	type ChallengePeriod = ChallengePeriod;
}

frame_support::parameter_types! {
	pub const ConfigDepositBase: Balance = 5 * COIN;
	pub const FriendDepositFactor: Balance = 50 * MILLI;
	pub const MaxFriends: u16 = 9;
	pub const RecoveryDeposit: Balance = 5 * COIN;
}
impl pallet_recovery::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Ring;
	type ConfigDepositBase = ConfigDepositBase;
	type FriendDepositFactor = FriendDepositFactor;
	type MaxFriends = MaxFriends;
	type RecoveryDeposit = RecoveryDeposit;
}

frame_support::parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80)
		* BlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 50;
}
impl pallet_scheduler::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
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
				// Specifically omitting the entire Balances pallet
				Call::Authorship(..) |
				Call::Staking(..) |
				Call::Offences(..) |
				Call::Session(..) |
				Call::Grandpa(..) |
				Call::ImOnline(..) |
				Call::AuthorityDiscovery(..) |
				Call::Council(..) |
				Call::TechnicalCommittee(..) |
				Call::ElectionsPhragmen(..) |
				Call::TechnicalMembership(..) |
				Call::Treasury(..) |
				Call::Democracy(..) |
				Call::Utility(..) |
				Call::Identity(..) |
				Call::Society(..) |
				Call::Recovery(pallet_recovery::Call::as_recovered(..)) |
				Call::Recovery(pallet_recovery::Call::vouch_recovery(..)) |
				Call::Recovery(pallet_recovery::Call::claim_recovery(..)) |
				Call::Recovery(pallet_recovery::Call::close_recovery(..)) |
				Call::Recovery(pallet_recovery::Call::remove_recovery(..)) |
				Call::Recovery(pallet_recovery::Call::cancel_recovered(..)) |
				Call::Vesting(darwinia_vesting::Call::vest(..)) |
				Call::Vesting(darwinia_vesting::Call::vest_other(..)) |
				// Specifically omitting Vesting `vested_transfer`, and `force_vested_transfer`
				Call::Scheduler(..) |
				Call::Proxy(..) |
				Call::Multisig(..) |
				// Specifically omitting the entire CrabBacking pallet
				// Specifically omitting the entire EthereumBacking pallet
				Call::EthereumRelay(..) |
				// Specifically omitting the entire TronBacking pallet
				Call::HeaderMMR(..) // Specifically omitting the entire EthereumRelayAuthorities pallet
			),
			ProxyType::Governance => matches!(
				c,
				Call::Council(..)
					| Call::TechnicalCommittee(..)
					| Call::ElectionsPhragmen(..)
					| Call::Treasury(..) | Call::Democracy(..)
					| Call::Utility(..)
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
frame_support::parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = deposit(0, 33);
	pub const MaxProxies: u16 = 32;
	pub const AnnouncementDepositBase: Balance = deposit(1, 8);
	pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
	pub const MaxPending: u16 = 32;
}
impl pallet_proxy::Config for Runtime {
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

frame_support::parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = deposit(0, 32);
	pub const MaxSignatories: u16 = 100;
}
impl pallet_multisig::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Ring;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = weights::pallet_multisig::WeightInfo<Runtime>;
}

frame_support::parameter_types! {
	pub const EthereumBackingModuleId: ModuleId = ModuleId(*b"da/ethbk");
	pub const EthereumBackingFeeModuleId: ModuleId = ModuleId(*b"da/ethfe");
	pub const RingLockLimit: Balance = 10_000_000 * COIN;
	pub const KtonLockLimit: Balance = 1_000 * COIN;
	// https://github.com/darwinia-network/darwinia-common/pull/377#issuecomment-730369387
	pub const AdvancedFee: Balance = 50 * COIN;
	pub const SyncReward: Balance = 1_000 * COIN;
}
impl darwinia_ethereum_backing::Config for Runtime {
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
frame_support::parameter_types! {
	pub const EthereumRelayAuthoritiesLockId: LockIdentifier = *b"ethrauth";
	pub const EthereumRelayAuthoritiesTermDuration: BlockNumber = 7 * DAYS;
	pub const MaxCandidates: usize = 7;
	pub const OpCodes: (OpCode, OpCode) = (
		[71, 159, 189, 249],
		[180, 188, 244, 151]
	);
	pub const SignThreshold: Perbill = Perbill::from_percent(60);
	pub const SubmitDuration: BlockNumber = 300;
}
impl darwinia_relay_authorities::Config<EthereumRelayAuthoritiesInstance> for Runtime {
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
frame_support::parameter_types! {
	pub const EthereumRelayModuleId: ModuleId = ModuleId(*b"da/ethrl");
	pub const EthereumNetwork: EthereumNetworkType = EthereumNetworkType::Mainnet;
	pub const ConfirmPeriod: BlockNumber = 3 * DAYS;
	pub const ApproveThreshold: Perbill = Perbill::from_percent(60);
	pub const RejectThreshold: Perbill = Perbill::from_percent(1);
}
impl darwinia_ethereum_relay::Config for Runtime {
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
frame_support::parameter_types! {
	pub const EthereumRelayerGameLockId: LockIdentifier = *b"da/rgame";
}
impl darwinia_relayer_game::Config<EthereumRelayerGameInstance> for Runtime {
	type RingCurrency = Ring;
	type LockId = EthereumRelayerGameLockId;
	type RingSlash = Treasury;
	type RelayerGameAdjustor = EthereumRelayerGameAdjustor;
	type RelayableChain = EthereumRelay;
	type WeightInfo = ();
}

frame_support::parameter_types! {
	pub const TronBackingModuleId: ModuleId = ModuleId(*b"da/trobk");
}
impl darwinia_tron_backing::Config for Runtime {
	type ModuleId = TronBackingModuleId;
	type RingCurrency = Ring;
	type KtonCurrency = Kton;
	type WeightInfo = ();
}

frame_support::parameter_types! {
	pub const CrabBackingModuleId: ModuleId = ModuleId(*b"da/crabk");
}
impl darwinia_crab_backing::Config for Runtime {
	type ModuleId = CrabBackingModuleId;
	type RingCurrency = Ring;
	type WeightInfo = ();
}

construct_runtime! {
	pub enum Runtime
	where
		Block = Block,
		NodeBlock = OpaqueBlock,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		// Basic stuff; balances is uncallable initially.
		System: frame_system::{Module, Call, Storage, Config, Event<T>} = 0,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Storage} = 1,

		// Must be before session.
		Babe: pallet_babe::{Module, Call, Storage, Config, Inherent, ValidateUnsigned} = 2,

		Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent} = 3,
		Balances: darwinia_balances::<Instance0>::{Module, Call, Storage, Config<T>, Event<T>} = 4,
		Kton: darwinia_balances::<Instance1>::{Module, Call, Storage, Config<T>, Event<T>} = 5,
		TransactionPayment: pallet_transaction_payment::{Module, Storage} = 6,

		// Consensus support.
		Authorship: pallet_authorship::{Module, Call, Storage} = 7,
		Staking: darwinia_staking::{Module, Call, Storage, Config<T>, Event<T>, ValidateUnsigned} = 8,
		Offences: pallet_offences::{Module, Call, Storage, Event} = 9,
		Historical: pallet_session_historical::{Module} = 10,
		Session: pallet_session::{Module, Call, Storage, Config<T>, Event} = 11,
		Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event, ValidateUnsigned} = 13,
		ImOnline: pallet_im_online::{Module, Call, Storage, Config<T>, Event<T>, ValidateUnsigned} = 14,
		AuthorityDiscovery: pallet_authority_discovery::{Module, Call, Config} = 15,
		HeaderMMR: darwinia_header_mmr::{Module, Call, Storage} = 35,

		// Governance stuff; uncallable initially.
		Council: pallet_collective::<Instance0>::{Module, Call, Storage, Origin<T>, Config<T>, Event<T>} = 16,
		TechnicalCommittee: pallet_collective::<Instance1>::{Module, Call, Storage, Origin<T>, Config<T>, Event<T>} = 17,
		ElectionsPhragmen: darwinia_elections_phragmen::{Module, Call, Storage, Config<T>, Event<T>} = 18,
		TechnicalMembership: pallet_membership::<Instance0>::{Module, Call, Storage, Config<T>, Event<T>} = 19,
		Treasury: darwinia_treasury::{Module, Call, Storage, Event<T>} = 20,
		Democracy: darwinia_democracy::{Module, Call, Storage, Config, Event<T>} = 37,

		// Vesting. Usable initially, but removed once all vesting is finished.
		Vesting: darwinia_vesting::{Module, Call, Storage, Event<T>, Config<T>} = 21,

		// Utility module.
		Utility: pallet_utility::{Module, Call, Event} = 22,

		// Less simple identity module.
		Identity: pallet_identity::{Module, Call, Storage, Event<T>} = 23,

		// Society module.
		Society: pallet_society::{Module, Call, Storage, Event<T>} = 24,

		// Social recovery module.
		Recovery: pallet_recovery::{Module, Call, Storage, Event<T>} = 25,

		// System scheduler.
		Scheduler: pallet_scheduler::{Module, Call, Storage, Event<T>} = 26,

		Sudo: pallet_sudo::{Module, Call, Storage, Config<T>, Event<T>} = 27,

		// Proxy module. Late addition.
		Proxy: pallet_proxy::{Module, Call, Storage, Event<T>} = 28,

		// Multisig module. Late addition.
		Multisig: pallet_multisig::{Module, Call, Storage, Event<T>} = 29,

		// Crab bridge.
		CrabBacking: darwinia_crab_backing::{Module, Storage} = 30,

		// Ethereum bridge.
		EthereumBacking: darwinia_ethereum_backing::{Module, Call, Storage, Config<T>, Event<T>} = 31,
		EthereumRelay: darwinia_ethereum_relay::{Module, Call, Storage, Config<T>, Event<T>} = 32,
		EthereumRelayerGame: darwinia_relayer_game::<Instance0>::{Module, Storage} = 33,
		EthereumRelayAuthorities: darwinia_relay_authorities::<Instance0>::{Module, Call, Storage, Event<T>} = 36,

		// Tron bridge.
		TronBacking: darwinia_tron_backing::{Module, Storage, Config<T>} = 34,
	}
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	Call: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: Call,
		public: <Signature as Verify>::Signer,
		account: AccountId,
		nonce: <Runtime as frame_system::Config>::Index,
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
		let address = <Runtime as frame_system::Config>::Lookup::unlookup(account);
		Some((call, (address, signature, extra)))
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

		fn current_epoch() -> sp_consensus_babe::Epoch {
			Babe::current_epoch()
		}

		fn next_epoch() -> sp_consensus_babe::Epoch {
			Babe::next_epoch()
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
		fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
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

pub struct CustomOnRuntimeUpgrade;
impl darwinia_elections_phragmen::migrations_2_0_0::ToV2 for CustomOnRuntimeUpgrade {
	type AccountId = AccountId;
	type Balance = Balance;
	type Module = ElectionsPhragmen;
}
impl frame_support::traits::OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		// --- substrate ---
		// use frame_support::migration::*;

		darwinia_elections_phragmen::migrations_2_0_0::apply::<Self>(5 * MILLI, 100 * MILLI)
	}
}
