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

// crates.io
use codec::{Encode, Decode};
// darwinia
use crate::*;
// polkadot
use xcm::latest::prelude::*;
// substrate
use frame_support::traits::Currency;
use sp_runtime::traits::Zero;
use sp_runtime::traits::Hash;

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
	pub const RelayNetwork: NetworkId = NetworkId::Westend;
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
		xcm_builder::WithComputedOrigin<
			(
				xcm_builder::AllowTopLevelPaidExecutionFrom<frame_support::traits::Everything>,
				// Parent and its exec plurality get free execution
				xcm_builder::AllowUnpaidExecutionFrom<
					darwinia_common_runtime::xcm_configs::ParentOrParentsExecutivePlurality,
				>,
				// Subscriptions for version tracking are OK.
				xcm_builder::AllowSubscriptionsFrom<
					darwinia_common_runtime::xcm_configs::ParentOrSiblings,
				>,
			),
			UniversalLocation,
			ConstU32<8>,
		>,
		// Expected responses are OK.
		xcm_builder::AllowKnownQueryResponses<PolkadotXcm>,
		// Subscriptions for version tracking are OK.
		xcm_builder::AllowSubscriptionsFrom<darwinia_common_runtime::xcm_configs::ParentOrSiblings>,
	),
>;

frame_support::parameter_types! {
	pub const MaxAssetsIntoHolding: u32 = 64;
	pub const MaxInstructions: u32 = 100;
	pub AnchoringSelfReserve: MultiLocation = MultiLocation::new(
		0,
		X1(PalletInstance(<Balances as frame_support::traits::PalletInfoAccess>::index() as u8))
	);
	pub UniversalLocation: InteriorMultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
	// One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
	pub UnitWeightCost: frame_support::weights::Weight = frame_support::weights::Weight::from_parts(1_000_000_000, 64 * 1024);
	pub LocalAssetsPalletLocation: MultiLocation = MultiLocation::new(
		0,
		X1(PalletInstance(<Assets as frame_support::traits::PalletInfoAccess>::index() as u8))
	);
	pub SelfLocation: MultiLocation = MultiLocation::here();
}

pub struct ToTreasury;
impl xcm_builder::TakeRevenue for ToTreasury {
	fn take_revenue(revenue: MultiAsset) {
		if let MultiAsset { id: Concrete(_location), fun: Fungible(amount) } = revenue {
			let treasury_account = Treasury::account_id();
			let _ = Balances::deposit_creating(&treasury_account, amount);

			frame_support::log::trace!(
				target: "xcm::weight",
				"LocalAssetTrader::to_treasury amount: {amount:?}, treasury: {treasury_account:?}"
			);
		}
	}
}

pub type XcmWeigher = xcm_builder::FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;

pub struct XcmExecutorConfig;
impl xcm_executor::Config for XcmExecutorConfig {
	type AssetClaims = PolkadotXcm;
	type AssetExchanger = ();
	type AssetLocker = ();
	// How to withdraw and deposit an asset.
	type AssetTransactor = LocalAssetTransactor;
	type AssetTrap = PolkadotXcm;
	type Barrier = Barrier;
	type CallDispatcher = RuntimeCall;
	type FeeManager = ();
	type IsReserve = xcm_builder::NativeAsset;
	type IsTeleporter = ();
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type MessageExporter = ();
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type ResponseHandler = PolkadotXcm;
	type RuntimeCall = RuntimeCall;
	type SafeCallFilter = frame_support::traits::Everything;
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
	type UniversalAliases = frame_support::traits::Nothing;
	// Teleporting is disabled.
	type UniversalLocation = UniversalLocation;
	type Weigher = XcmWeigher;
	type XcmSender = XcmRouter;
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation =
	xcm_primitives::SignedToAccountId20<RuntimeOrigin, AccountId, RelayNetwork>;
/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

#[cfg(feature = "runtime-benchmarks")]
frame_support::parameter_types! {
	pub ReachableDest: Option<MultiLocation> = Some(Parent.into());
}

impl pallet_xcm::Config for Runtime {
	// ^ Override for AdvertisedXcmVersion default
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type ExecuteXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type MaxLockers = ConstU32<8>;
	#[cfg(feature = "runtime-benchmarks")]
	type ReachableDest = ReachableDest;
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

pub struct EthereumXcmEnsureProxy;
impl xcm_primitives::EnsureProxy<AccountId> for EthereumXcmEnsureProxy {
	fn ensure_ok(delegator: AccountId, delegatee: AccountId) -> Result<(), &'static str> {
		// The EVM implicitely contains an Any proxy, so we only allow for "Any" proxies
		let def: pallet_proxy::ProxyDefinition<AccountId, pallets::proxy::ProxyType, BlockNumber> =
			pallet_proxy::Pallet::<Runtime>::find_proxy(
				&delegator,
				&delegatee,
				Some(pallets::proxy::ProxyType::Any),
			)
			.map_err(|_| "proxy error: expected `ProxyType::Any`")?;
		// We only allow to use it for delay zero proxies, as the call will immediatly be executed
		frame_support::ensure!(def.delay.is_zero(), "proxy delay is Non-zero`");
		Ok(())
	}
}

impl pallet_ethereum_xcm::Config for Runtime {
	type InvalidEvmTransactionError = pallet_ethereum::InvalidTransactionWrapper;
	type ValidatedTransaction = pallet_ethereum::ValidatedTransaction<Self>;
	type XcmEthereumOrigin = pallet_ethereum_xcm::EnsureXcmEthereumTransaction;
	type ReservedXcmpWeight = <Runtime as cumulus_pallet_parachain_system::Config>::ReservedXcmpWeight;
	type EnsureProxy = EthereumXcmEnsureProxy;
	type ControllerOrigin = frame_system::EnsureRoot<AccountId>;
}

// For now we only allow to transact in the relay, although this might change in the future
// Transactors just defines the chains in which we allow transactions to be issued through
// xcm
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, scale_info::TypeInfo)]
pub enum Transactors {
	Relay,
}

// Default for benchmarking
#[cfg(feature = "runtime-benchmarks")]
impl Default for Transactors {
	fn default() -> Self {
		Transactors::Relay
	}
}

impl TryFrom<u8> for Transactors {
	type Error = ();
	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
			0u8 => Ok(Transactors::Relay),
			_ => Err(()),
		}
	}
}

impl xcm_primitives::UtilityEncodeCall for Transactors {
	fn encode_call(self, call: xcm_primitives::UtilityAvailableCalls) -> Vec<u8> {
		match self {
			// The encoder should be polkadot
			Transactors::Relay => {
				moonbeam_relay_encoder::polkadot::PolkadotEncoder.encode_call(call)
			}
		}
	}
}

impl xcm_primitives::XcmTransact for Transactors {
	fn destination(self) -> MultiLocation {
		match self {
			Transactors::Relay => MultiLocation::parent(),
		}
	}
}

// Our AssetType. For now we only handle Xcm Assets
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, scale_info::TypeInfo)]
pub enum AssetType {
	Xcm(MultiLocation),
}
impl Default for AssetType {
	fn default() -> Self {
		Self::Xcm(MultiLocation::here())
	}
}

impl From<MultiLocation> for AssetType {
	fn from(location: MultiLocation) -> Self {
		Self::Xcm(location)
	}
}

impl Into<Option<MultiLocation>> for AssetType {
	fn into(self) -> Option<MultiLocation> {
		match self {
			Self::Xcm(location) => Some(location),
		}
	}
}

// Implementation on how to retrieve the AssetId from an AssetType
// We take it
impl From<AssetType> for crate::AssetId {
	fn from(asset: AssetType) -> crate::AssetId {
		match asset {
			AssetType::Xcm(id) => {
				let mut result: [u8; 8] = [0u8; 8];
				let hash: sp_core::H256 = id.using_encoded(<Runtime as frame_system::Config>::Hashing::hash);
				result.copy_from_slice(&hash.as_fixed_bytes()[0..8]);
				u64::from_le_bytes(result)
			}
		}
	}
}

// Our currencyId. We distinguish for now between SelfReserve, and Others, defined by their Id.
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, scale_info::TypeInfo)]
pub enum CurrencyId {
	SelfReserve,
	// ForeignAsset(crate::AssetId),
	// // Our local assets
	// LocalAssetReserve(crate::AssetId),
}

impl xcm_primitives::AccountIdToCurrencyId<AccountId, CurrencyId> for Runtime {
	fn account_to_currency_id(account: AccountId) -> Option<CurrencyId> {
		match account {
			// TODO this should be our BalancesPrecompile address
			// the self-reserve currency is identified by the pallet-balances address
			a if a == sp_core::H160::from_low_u64_be(2050).into() => Some(CurrencyId::SelfReserve),
			// the rest of the currencies, by their corresponding erc20 address
			_ => {
				unimplemented!("todo");
			}
			// _ => Runtime::account_to_asset_id(account).map(|(prefix, asset_id)| {
			// 	CurrencyId::LocalAssetReserve(asset_id)
			// 	// We don't have ForeignAsset
			// 	if prefix == FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX.to_vec() {
			// 		CurrencyId::ForeignAsset(asset_id)
			// 	} else {
			// 		CurrencyId::LocalAssetReserve(asset_id)
			// 	}
			// }),
		}
	}
}

// How to convert from CurrencyId to MultiLocation
pub struct CurrencyIdtoMultiLocation;
impl sp_runtime::traits::Convert<CurrencyId, Option<xcm::opaque::latest::MultiLocation>>
	for CurrencyIdtoMultiLocation
{
	fn convert(currency: CurrencyId) -> Option<MultiLocation> {
		match currency {
			CurrencyId::SelfReserve => {
				let multi: MultiLocation = AnchoringSelfReserve::get();
				Some(multi)
			}
			// CurrencyId::ForeignAsset(asset) => AssetXConverter::reverse_ref(asset).ok(),
			// // No transactor matches this yet, so even if we have this enum variant the transfer will fail
			// CurrencyId::LocalAssetReserve(asset) => {
			// 	let mut location = LocalAssetsPalletLocation::get();
			// 	location.push_interior(xcm::opaque::latest::Junction::GeneralIndex(asset.into())).ok();
			// 	Some(location)
			// }
		}
	}
}

// We use all transactors
// These correspond to
// SelfReserve asset, both pre and post 0.9.16
// Foreign assets
// Local assets, both pre and post 0.9.16
// We can remove the Old reanchor once
// we import https://github.com/open-web3-stack/open-runtime-module-library/pull/708
pub type AssetTransactors = (
	LocalAssetTransactor,
);

impl pallet_xcm_transactor::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Transactor = Transactors;
	type DerivativeAddressRegistrationOrigin = frame_system::EnsureRoot<AccountId>;
	type SovereignAccountDispatcherOrigin = frame_system::EnsureRoot<AccountId>;
	type CurrencyId = CurrencyId;
	type AccountIdToMultiLocation = xcm_primitives::AccountIdToMultiLocation<AccountId>;
	type CurrencyIdToMultiLocation = CurrencyIdtoMultiLocation;
	type XcmSender = XcmRouter;
	type SelfLocation = SelfLocation;
	type Weigher = XcmWeigher;
	type LocationInverter = xcm_builder::LocationInverter<Ancestry>;
	type BaseXcmWeight = BaseXcmWeight;
	type AssetTransactor = AssetTransactors;
	type ReserveProvider = xcm_primitives::AbsoluteAndRelativeReserve<Ancestry>;
	type WeightInfo = pallet_xcm_transactor::weights::SubstrateWeight<Runtime>;
}
