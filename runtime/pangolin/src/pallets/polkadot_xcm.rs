// This file is part of Darwinia.
//
// Copyright (C) 2018-2023 Darwinia Network
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
// substrate
use frame_support::traits::Currency;

/// Means for transacting assets on this chain.
pub type LocalAssetTransactor = xcm_builder::CurrencyAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	xcm_builder::IsConcrete<AnchoringSelfReserve>,
	// Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports.
	(),
>;

frame_support::parameter_types! {
	pub const RelayNetwork: xcm::latest::prelude::NetworkId = xcm::latest::prelude::NetworkId::Any;
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
}
/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
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
	darwinia_common_runtime::xcm_configs::Account20Hash<AccountId>,
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

pub type Barrier = darwinia_common_runtime::xcm_configs::DenyThenTry<
	darwinia_common_runtime::xcm_configs::DenyReserveTransferToRelayChain,
	(
		xcm_builder::TakeWeightCredit,
		xcm_builder::AllowTopLevelPaidExecutionFrom<frame_support::traits::Everything>,
		// Parent and its exec plurality get free execution
		xcm_builder::AllowUnpaidExecutionFrom<
			darwinia_common_runtime::xcm_configs::ParentOrParentsExecutivePlurality,
		>,
		// Expected responses are OK.
		xcm_builder::AllowKnownQueryResponses<PolkadotXcm>,
		// Subscriptions for version tracking are OK.
		xcm_builder::AllowSubscriptionsFrom<darwinia_common_runtime::xcm_configs::ParentOrSiblings>,
	),
>;

frame_support::parameter_types! {
	pub const MaxInstructions: u32 = 100;
	pub AnchoringSelfReserve: xcm::latest::prelude::MultiLocation = xcm::latest::prelude::MultiLocation::new(
		0,
		xcm::latest::prelude::X1(xcm::latest::prelude::PalletInstance(<Balances as frame_support::traits::PalletInfoAccess>::index() as u8))
	);
	// One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
	pub Ancestry: xcm::latest::prelude::MultiLocation = xcm::latest::prelude::Parachain(ParachainInfo::parachain_id().into()).into();
	pub UnitWeightCost: u64 = 1_000_000_000;
}

pub struct ToTreasury;
impl xcm_builder::TakeRevenue for ToTreasury {
	fn take_revenue(revenue: xcm::latest::prelude::MultiAsset) {
		if let xcm::latest::prelude::MultiAsset {
			id: xcm::latest::prelude::Concrete(_location),
			fun: xcm::latest::prelude::Fungible(amount),
		} = revenue
		{
			let treasury_account = Treasury::account_id();
			let _ = Balances::deposit_creating(&treasury_account, amount);

			frame_support::log::trace!(
				target: "xcm::weight",
				"LocalAssetTrader::to_treasury amount: {amount:?}, treasury: {treasury_account:?}"
			);
		}
	}
}

pub struct XcmExecutorConfig;
impl xcm_executor::Config for XcmExecutorConfig {
	type AssetClaims = PolkadotXcm;
	// How to withdraw and deposit an asset.
	type AssetTransactor = LocalAssetTransactor;
	type AssetTrap = PolkadotXcm;
	type Barrier = Barrier;
	type IsReserve = xcm_builder::NativeAsset;
	type IsTeleporter = ();
	// Teleporting is disabled.
	type LocationInverter = xcm_builder::LocationInverter<Ancestry>;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type ResponseHandler = PolkadotXcm;
	type RuntimeCall = RuntimeCall;
	type SubscriptionService = PolkadotXcm;
	type Trader = xcm_configs::LocalAssetTrader<
		frame_support::weights::ConstantMultiplier<
			Balance,
			darwinia_common_runtime::xcm_configs::XcmBaseWeightFee,
		>,
		AnchoringSelfReserve,
		AccountId,
		Balances,
		DealWithFees<Runtime>,
		ToTreasury,
	>;
	type Weigher = xcm_builder::FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type XcmSender = XcmRouter;
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation =
	xcm_primitives::SignedToAccountId20<RuntimeOrigin, AccountId, RelayNetwork>;
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
	type ExecuteXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type LocationInverter = xcm_builder::LocationInverter<Ancestry>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SendXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type Weigher = xcm_builder::FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
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
