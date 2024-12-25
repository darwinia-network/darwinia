// This file is part of Darwinia.
//
// Copyright (C) Darwinia Network
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

// darwinia
use crate::{AssetId, Assets, *};
// polkadot-sdk
use xcm::latest::prelude::*;

frame_support::parameter_types! {
	pub const RelayLocation: Location = Location::parent();
	pub const RelayNetwork: NetworkId = NetworkId::Polkadot;
	pub const MaxAssetsIntoHolding: u32 = 64;
	pub const MaxInstructions: u32 = 100;
	/// A temporary weight value for each XCM instruction.
	/// NOTE: This should be removed after we account for PoV weights.
	pub const TempFixedXcmWeight: frame_support::weights::Weight = frame_support::weights::Weight::from_parts(1_000_000_000, 0);
	pub SelfReserve: Location = Location::new(
		0,
		[PalletInstance(<Balances as frame_support::traits::PalletInfoAccess>::index() as u8)]
	);
	pub UniversalLocation: InteriorLocation =
		[GlobalConsensus(RelayNetwork::get()), Parachain(ParachainInfo::parachain_id().into())].into();
	/// The amount of weight an XCM operation takes. This is a safe overestimate.
	pub BaseXcmWeight: frame_support::weights::Weight = frame_support::weights::Weight::from_parts(1_000_000_000, 1024);
	/// Xcm fees will go to the treasury account
	pub XcmFeesAccount: AccountId = Treasury::account_id();
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
}

/// The transactor for our own chain currency.
pub type LocalAssetTransactor = xcm_builder::FungibleAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching any of the locations in
	// SelfReserveRepresentations
	xcm_builder::IsConcrete<SelfReserve>,
	// We can convert the MultiLocations with our converter above:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We dont allow teleport
	(),
>;
// The non-reserve fungible transactor type
// It will use pallet-assets, and the Id will be matched against AsAssetType
pub type ForeignFungiblesTransactor = xcm_builder::FungiblesAdapter<
	// Use this fungibles implementation:
	Assets,
	// Use this currency when it is a fungible asset matching the given location or name:
	(
		xcm_builder::ConvertedConcreteId<
			AssetId,
			Balance,
			xcm_primitives::AsAssetType<AssetId, AssetType, AssetManager>,
			xcm_executor::traits::JustTry,
		>,
	),
	// Do a simple punn to convert an AccountId20 Location into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We dont allow teleports.
	xcm_builder::NoChecking,
	// We dont track any teleports
	(),
>;
pub type AssetTransactors = (LocalAssetTransactor, ForeignFungiblesTransactor);
/// Type for specifying how a `Location` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the parent `AccountId`.
	xcm_builder::ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	xcm_builder::SiblingParachainConvertsVia<polkadot_parachain::primitives::Sibling, AccountId>,
	// Straight up local `AccountId20` origins just alias directly to `AccountId`.
	xcm_builder::AccountKey20Aliases<RelayNetwork, AccountId>,
	// The rest of locations are converted via hashing it.
	xcm_config::Account20Hash<AccountId>,
);
/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	xcm_builder::SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognized.
	xcm_builder::RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognized.
	xcm_builder::SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	// Native signed account converter; this just converts an `AccountKey20` origin into a normal
	// `RuntimeOrigin::Signed` origin of the same 20-byte value.
	xcm_builder::SignedAccountKey20AsNative<RelayNetwork, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	pallet_xcm::XcmPassthrough<RuntimeOrigin>,
);

pub type Barrier = xcm_builder::TrailingSetTopicAsId<(
	xcm_builder::TakeWeightCredit,
	xcm_builder::WithComputedOrigin<
		(
			// If the message is one that immediately attempts to pay for execution, then
			// allow it.
			xcm_builder::AllowTopLevelPaidExecutionFrom<frame_support::traits::Everything>,
			// Parent, its pluralities (i.e. governance bodies), and the Fellows plurality
			// get free execution.
			xcm_builder::AllowExplicitUnpaidExecutionFrom<xcm_config::ParentOrParentsPlurality>,
			// Subscriptions for version tracking are OK.
			xcm_builder::AllowSubscriptionsFrom<xcm_config::ParentRelayOrSiblingParachains>,
			// HRMP notifications from the relay chain are OK.
			xcm_builder::AllowHrmpNotificationsFromRelayChain,
		),
		UniversalLocation,
		ConstU32<8>,
	>,
	// Expected responses are OK.
	xcm_builder::AllowKnownQueryResponses<PolkadotXcm>,
)>;

/// This is the struct that will handle the revenue from xcm fees
/// We do not burn anything because we want to mimic exactly what
/// the sovereign account has
pub type XcmFeesToAccount = xcm_primitives::XcmFeesToAccount<
	Assets,
	(
		xcm_builder::ConvertedConcreteId<
			AssetId,
			Balance,
			xcm_primitives::AsAssetType<AssetId, AssetType, AssetManager>,
			xcm_executor::traits::JustTry,
		>,
	),
	AccountId,
	XcmFeesAccount,
>;
pub type XcmWeigher = xcm_builder::FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation =
	xcm_primitives::SignedToAccountId20<RuntimeOrigin, AccountId, RelayNetwork>;
/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = xcm_builder::WithUniqueTopic<(
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
)>;

pub struct XcmExecutorConfig;
impl xcm_executor::Config for XcmExecutorConfig {
	type Aliasers = frame_support::traits::Nothing;
	type AssetClaims = PolkadotXcm;
	type AssetExchanger = ();
	type AssetLocker = ();
	// How to withdraw and deposit an asset.
	type AssetTransactor = AssetTransactors;
	type AssetTrap = PolkadotXcm;
	type Barrier = Barrier;
	type CallDispatcher = RuntimeCall;
	type FeeManager = ();
	type HrmpChannelAcceptedHandler = ();
	type HrmpChannelClosingHandler = ();
	type HrmpNewChannelOpenRequestHandler = ();
	type IsReserve = orml_xcm_support::MultiNativeAsset<
		xcm_primitives::AbsoluteAndRelativeReserve<SelfLocationAbsolute>,
	>;
	type IsTeleporter = ();
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type MessageExporter = ();
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type ResponseHandler = PolkadotXcm;
	type RuntimeCall = RuntimeCall;
	type SafeCallFilter = frame_support::traits::Everything;
	type SubscriptionService = PolkadotXcm;
	type Trader = (
		xcm_builder::UsingComponents<
			<Runtime as pallet_transaction_payment::Config>::WeightToFee,
			SelfReserve,
			AccountId,
			Balances,
			DealWithFees<Runtime>,
		>,
		xcm_primitives::FirstAssetTrader<AssetType, AssetManager, XcmFeesToAccount>,
	);
	type TransactionalProcessor = xcm_builder::FrameTransactionalProcessor;
	type UniversalAliases = frame_support::traits::Nothing;
	// Teleporting is disabled.
	type UniversalLocation = UniversalLocation;
	type Weigher = xcm_builder::FixedWeightBounds<TempFixedXcmWeight, RuntimeCall, MaxInstructions>;
	type XcmRecorder = PolkadotXcm;
	type XcmSender = XcmRouter;
}

impl pallet_xcm::Config for Runtime {
	type AdminOrigin = RootOr<GeneralAdmin>;
	// ^ Override for AdvertisedXcmVersion default
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type ExecuteXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type MaxLockers = ConstU32<8>;
	type MaxRemoteLockConsumers = ();
	type RemoteLockConsumerIdentifier = ();
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SendXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type SovereignAccountOf = LocationToAccountId;
	type TrustedLockers = ();
	type UniversalLocation = UniversalLocation;
	type Weigher = XcmWeigher;
	type WeightInfo = pallet_xcm::TestWeightInfo;
	type XcmExecuteFilter = frame_support::traits::Everything;
	type XcmExecutor = xcm_executor::XcmExecutor<XcmExecutorConfig>;
	type XcmReserveTransferFilter = frame_support::traits::Everything;
	type XcmRouter = XcmRouter;
	type XcmTeleportFilter = frame_support::traits::Nothing;

	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = xcm_executor::XcmExecutor<XcmExecutorConfig>;
}
