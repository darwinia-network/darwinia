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
use xcm::prelude::*;

frame_support::parameter_types! {
	/// 1000 is AssetHub paraId.
	/// 50 is pallet-assets index on AssetHub.
	/// 1984 is the id of USDT on AssetHub(Polkadot).
	pub UsdtLocation: xcm::v3::Location = xcm::v3::Location::new(
		1,
		xcm::v3::prelude::X3(xcm::v3::prelude::Parachain(1000), xcm::v3::prelude::PalletInstance(50), xcm::v3::prelude::GeneralIndex(1984))
	);
	/// 23 is the id of PINK on AssetHub(Polkadot).
	pub PinkLocation: xcm::v3::Location = xcm::v3::Location::new(
		1,
		xcm::v3::prelude::X3(xcm::v3::prelude::Parachain(1000), xcm::v3::prelude::PalletInstance(50), xcm::v3::prelude::GeneralIndex(23))
	);
	/// Relaychain native token DOT
	pub DotLocation: MultiLocation = MultiLocation::parent();
}

// We instruct how to register the Assets
// In this case, we tell it to create an Asset in pallet-assets
pub struct AssetRegistrar;
impl pallet_asset_manager::AssetRegistrar<Runtime> for AssetRegistrar {
	#[frame_support::transactional]
	fn create_foreign_asset(
		asset: AssetId,
		min_balance: Balance,
		metadata: xcm_config::AssetRegistrarMetadata,
		is_sufficient: bool,
	) -> sp_runtime::DispatchResult {
		Assets::force_create(
			RuntimeOrigin::root(),
			asset.into(),
			AssetManager::account_id(),
			is_sufficient,
			min_balance,
		)?;

		// Lastly, the metadata
		Assets::force_set_metadata(
			RuntimeOrigin::root(),
			asset.into(),
			metadata.name,
			metadata.symbol,
			metadata.decimals,
			metadata.is_frozen,
		)
	}

	#[frame_support::transactional]
	fn destroy_foreign_asset(asset: AssetId) -> sp_runtime::DispatchResult {
		// Mark the asset as destroying
		Assets::start_destroy(RuntimeOrigin::root(), asset.into())?;

		Ok(())
	}

	fn destroy_asset_dispatch_info_weight(asset: AssetId) -> frame_support::weights::Weight {
		// substrate
		use frame_support::dispatch::GetDispatchInfo;

		// The dispatch info of destroy
		let call_weight =
			RuntimeCall::Assets(pallet_assets::Call::<Runtime>::start_destroy { id: asset.into() })
				.get_dispatch_info()
				.weight;

		// The db write for removing the precompile revert code in the EVM.
		call_weight.saturating_add(<Runtime as frame_system::Config>::DbWeight::get().writes(1))
	}
}

// Our AssetType. For now we only handle Xcm Assets
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum AssetType {
	Xcm(xcm::v3::Location),
}
impl Default for AssetType {
	fn default() -> Self {
		Self::Xcm(xcm::v3::Location::here())
	}
}
impl From<xcm::v3::Location> for AssetType {
	fn from(location: xcm::v3::Location) -> Self {
		Self::Xcm(location)
	}
}
// Implementation on how to retrieve the AssetId from an `AssetType`.
// We simply hash the `AssetType` and take the lowest 128 bits.
impl From<AssetType> for crate::AssetId {
	fn from(asset: AssetType) -> crate::AssetId {
		use sp_runtime::traits::Hash;

		match asset {
			AssetType::Xcm(id) if id == UsdtLocation::get() => 1027,
			AssetType::Xcm(id) if id == PinkLocation::get() => 1028,
			AssetType::Xcm(id) if id == DotLocation::get() => 1029,
			AssetType::Xcm(id) => {
				use sp_runtime::traits::Hash;

				let mut result: [u8; 8] = [0_u8; 8];
				let hash: sp_core::H256 = id.using_encoded(dc_primitives::Hashing::hash);
				result.copy_from_slice(&hash.as_fixed_bytes()[0..8]);

					u64::from_le_bytes(result)
				},
		}
	}
}
#[allow(clippy::from_over_into)]
impl Into<Option<xcm::v3::Location>> for AssetType {
	fn into(self) -> Option<xcm::v3::Location> {
		match self {
			Self::Xcm(location) => Some(location),
		}
	}
}
// This can be removed once we fully adopt xcm::v4 everywhere
impl TryFrom<Location> for AssetType {
	type Error = ();

	fn try_from(location: Location) -> Result<Self, Self::Error> {
		Ok(Self::Xcm(location.try_into()?))
	}
}

impl pallet_asset_manager::Config for Runtime {
	type AssetId = AssetId;
	type AssetRegistrar = AssetRegistrar;
	type AssetRegistrarMetadata = xcm_config::AssetRegistrarMetadata;
	type Balance = Balance;
	type ForeignAssetModifierOrigin = RootOr<GeneralAdmin>;
	type ForeignAssetType = AssetType;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_asset_manager::WeightInfo<Self>;
}
