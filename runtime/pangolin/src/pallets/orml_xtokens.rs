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
// polkadot
use xcm::latest::prelude::*;

// Our currencyId. We distinguish for now between SelfReserve, and Others, defined by their Id.
#[derive(
	Clone, Debug, PartialEq, Eq, PartialOrd, Ord, codec::Encode, codec::Decode, scale_info::TypeInfo,
)]
pub enum CurrencyId {
	// Our native token
	SelfReserve,
	// Assets representing other chains native tokens
	ForeignAsset(crate::AssetId),
}

// How to convert from CurrencyId to MultiLocation
pub struct CurrencyIdtoMultiLocation<AssetXConverter>(core::marker::PhantomData<AssetXConverter>);
impl<AssetXConverter> sp_runtime::traits::Convert<CurrencyId, Option<MultiLocation>>
	for CurrencyIdtoMultiLocation<AssetXConverter>
where
	AssetXConverter: sp_runtime::traits::MaybeEquivalence<MultiLocation, crate::AssetId>,
{
	fn convert(currency: CurrencyId) -> Option<MultiLocation> {
		match currency {
			CurrencyId::SelfReserve => {
				let multi: MultiLocation = pallets::polkadot_xcm::AnchoringSelfReserve::get();
				Some(multi)
			},
			CurrencyId::ForeignAsset(asset) => AssetXConverter::convert_back(&asset),
		}
	}
}

frame_support::parameter_types! {
	pub const MaxAssetsForTransfer: usize = 2;

	// This is how we are going to detect whether the asset is a Reserve asset
	// This however is the chain part only
	pub SelfLocation: MultiLocation = MultiLocation::here();
	// We need this to be able to catch when someone is trying to execute a non-
	// cross-chain transfer in xtokens through the absolute path way
	pub SelfLocationAbsolute: MultiLocation = MultiLocation {
		parents:1,
		interior: Junctions::X1(
			Parachain(ParachainInfo::parachain_id().into())
		)
	};
}

impl orml_xtokens::Config for Runtime {
	type AccountIdToMultiLocation = xcm_primitives::AccountIdToMultiLocation<AccountId>;
	type Balance = Balance;
	type BaseXcmWeight = BaseXcmWeight;
	type CurrencyId = CurrencyId;
	type CurrencyIdConvert = CurrencyIdtoMultiLocation<
		xcm_primitives::AsAssetType<
			crate::AssetId,
			pallets::asset_manager::AssetType,
			AssetManager,
		>,
	>;
	type MaxAssetsForTransfer = MaxAssetsForTransfer;
	// We don't have this case: fee_reserve != non_fee_reserve
	type MinXcmFee = orml_xcm_support::DisabledParachainFee;
	type MultiLocationsFilter = frame_support::traits::Everything;
	type ReserveProvider = xcm_primitives::AbsoluteAndRelativeReserve<SelfLocationAbsolute>;
	type RuntimeEvent = RuntimeEvent;
	type SelfLocation = SelfLocation;
	type UniversalLocation = UniversalLocation;
	type Weigher = pallets::polkadot_xcm::XcmWeigher;
	type XcmExecutor = xcm_executor::XcmExecutor<pallets::polkadot_xcm::XcmExecutorConfig>;
}
