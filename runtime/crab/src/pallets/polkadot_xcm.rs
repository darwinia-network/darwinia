// This file is part of Darwinia.
//
// Copyright (C) 2018-2022 Darwinia Network
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
use crate::*;
use darwinia_common_runtime::xcm_barrier::*;
// moonbeam
use xcm_primitives::*;
// polkadot
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use xcm::latest::prelude::*;
use xcm_builder::*;
use xcm_executor::XcmExecutor;
// substrate
use frame_support::traits::{Everything, Nothing, PalletInfoAccess};

/// Means for transacting assets on this chain.
pub type LocalAssetTransactor = CurrencyAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<AnchoringSelfReserve>,
	// Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports.
	(),
>;

frame_support::parameter_types! {
	pub const RelayNetwork: NetworkId = NetworkId::Kusama;
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
}
/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the parent `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId20` origins just alias directly to `AccountId`.
	AccountKey20Aliases<RelayNetwork, AccountId>,
);
/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognized.
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognized.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	// Native signed account converter; this just converts an `AccountKey20` origin into a normal
	// `RuntimeOrigin::Signed` origin of the same 20-byte value.
	SignedAccountKey20AsNative<RelayNetwork, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<RuntimeOrigin>,
);

pub type Barrier = DenyThenTry<
	DenyReserveTransferToRelayChain,
	(
		TakeWeightCredit,
		AllowTopLevelPaidExecutionFrom<Everything>,
		// Parent and its exec plurality get free execution
		AllowUnpaidExecutionFrom<ParentOrParentsExecutivePlurality>,
		// Expected responses are OK.
		AllowKnownQueryResponses<PolkadotXcm>,
		// Subscriptions for version tracking are OK.
		AllowSubscriptionsFrom<ParentOrSiblings>,
	),
>;

frame_support::parameter_types! {
	pub const MaxInstructions: u32 = 100;
	pub AnchoringSelfReserve: MultiLocation = MultiLocation::new(
		0,
		X1(PalletInstance(<Balances as PalletInfoAccess>::index() as u8))
	);
	// One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
	pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
	pub UnitWeightCost: u64 = 1_000_000_000;
}

pub struct XcmExecutorConfig;
impl xcm_executor::Config for XcmExecutorConfig {
	type AssetClaims = PolkadotXcm;
	// How to withdraw and deposit an asset.
	type AssetTransactor = LocalAssetTransactor;
	type AssetTrap = PolkadotXcm;
	type Barrier = Barrier;
	type IsReserve = NativeAsset;
	type IsTeleporter = ();
	// Teleporting is disabled.
	type LocationInverter = LocationInverter<Ancestry>;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type ResponseHandler = PolkadotXcm;
	type RuntimeCall = RuntimeCall;
	type SubscriptionService = PolkadotXcm;
	type Trader = UsingComponents<
		WeightToFee,
		AnchoringSelfReserve,
		AccountId,
		Balances,
		DealWithFees<Runtime>,
	>;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type XcmSender = XcmRouter;
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId20<RuntimeOrigin, AccountId, RelayNetwork>;
/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

impl pallet_xcm::Config for Runtime {
	// ^ Override for AdvertisedXcmVersion default
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type LocationInverter = LocationInverter<Ancestry>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type XcmExecuteFilter = Nothing;
	// ^ Disable dispatchable execute on the XCM pallet.
	// Needs to be `Everything` for local testing.
	type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
	type XcmReserveTransferFilter = Nothing;
	type XcmRouter = XcmRouter;
	type XcmTeleportFilter = Nothing;

	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmExecutorConfig>;
}
