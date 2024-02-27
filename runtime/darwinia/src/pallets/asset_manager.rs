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
use codec::{Decode, Encode};
// darwinia
use crate::*;
// polkadot
use xcm::prelude::*;

// We instruct how to register the Assets
// In this case, we tell it to create an Asset in pallet-assets
pub struct AssetRegistrar;
impl pallet_asset_manager::AssetRegistrar<Runtime> for AssetRegistrar {
	#[frame_support::transactional]
	fn create_foreign_asset(
		asset: crate::AssetId,
		min_balance: Balance,
		metadata: xcm_configs::AssetRegistrarMetadata,
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
	fn destroy_foreign_asset(asset: crate::AssetId) -> sp_runtime::DispatchResult {
		// Mark the asset as destroying
		Assets::start_destroy(RuntimeOrigin::root(), asset.into())?;
		Ok(())
	}

	fn destroy_asset_dispatch_info_weight(asset: crate::AssetId) -> frame_support::weights::Weight {
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

pub struct LocalAssetIdCreator;
impl pallet_asset_manager::LocalAssetIdCreator<Runtime> for LocalAssetIdCreator {
	fn create_asset_id_from_metadata(_local_asset_counter: u128) -> crate::AssetId {
		// We don't need to create local asset.
		0
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
// Implementation on how to retrieve the AssetId from an `AssetType`.
// We simply hash the `AssetType` and take the lowest 128 bits.
impl From<AssetType> for crate::AssetId {
	fn from(asset: AssetType) -> crate::AssetId {
		use sp_runtime::traits::Hash;

		match asset {
			AssetType::Xcm(id) =>
				if id == UsdtLocation::get() {
					1027
				} else if id == PinkLocation::get() {
					1028
				} else {
					let mut result: [u8; 8] = [0_u8; 8];
					let hash: sp_core::H256 = id.using_encoded(dc_primitives::Hashing::hash);

					result.copy_from_slice(&hash.as_fixed_bytes()[0..8]);

					u64::from_le_bytes(result)
				},
		}
	}
}
#[allow(clippy::from_over_into)]
impl Into<Option<MultiLocation>> for AssetType {
	fn into(self) -> Option<MultiLocation> {
		match self {
			Self::Xcm(location) => Some(location),
		}
	}
}

frame_support::parameter_types! {
	/// 1000 is AssetHub paraId.
	/// 50 is pallet-assets index on AssetHub.
	/// 1984 is the id of USDT on AssetHub(Polkadot).
	pub UsdtLocation: MultiLocation = MultiLocation::new(
		1,
		X3(Parachain(1000), PalletInstance(50), GeneralIndex(1984))
	);

	/// 23 is the id of PINK on AssetHub(Polkadot).
	pub PinkLocation: MultiLocation = MultiLocation::new(
		1,
		X3(Parachain(1000), PalletInstance(50), GeneralIndex(23))
	);
}

impl pallet_asset_manager::Config for Runtime {
	type AssetId = crate::AssetId;
	type AssetRegistrar = AssetRegistrar;
	type AssetRegistrarMetadata = xcm_configs::AssetRegistrarMetadata;
	type Balance = Balance;
	type Currency = Balances;
	type ForeignAssetModifierOrigin = Root;
	type ForeignAssetType = AssetType;
	type LocalAssetDeposit = ConstU128<0>;
	type LocalAssetIdCreator = LocalAssetIdCreator;
	type LocalAssetModifierOrigin = frame_system::EnsureNever<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_asset_manager::weights::SubstrateWeight<Runtime>;
}
