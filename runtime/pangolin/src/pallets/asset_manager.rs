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
use frame_support::{dispatch::GetDispatchInfo, pallet_prelude::*};

// We instruct how to register the Assets
// In this case, we tell it to Create an Asset in pallet-assets
pub struct AssetRegistrar;
use frame_support::{pallet_prelude::DispatchResult, transactional};

impl pallet_asset_manager::AssetRegistrar<Runtime> for AssetRegistrar {
	#[transactional]
	fn create_foreign_asset(
		asset: crate::AssetId,
		min_balance: Balance,
		metadata: xcm_configs::AssetRegistrarMetadata,
		is_sufficient: bool,
	) -> DispatchResult {
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

	#[transactional]
	fn destroy_foreign_asset(asset: crate::AssetId) -> DispatchResult {
		// Mark the asset as destroying
		Assets::start_destroy(RuntimeOrigin::root(), asset.into())?;
		Ok(())
	}

	fn destroy_asset_dispatch_info_weight(asset: crate::AssetId) -> Weight {
		// For us both of them (Foreign and Local) have the same annotated weight for a given
		// witness
		// We need to take the dispatch info from the destroy call, which is already annotated in
		// the assets pallet
		// Additionally, we need to add a DB write for removing the precompile revert code in the
		// EVM

		// This is the dispatch info of destroy
		let call_weight =
			RuntimeCall::Assets(pallet_assets::Call::<Runtime>::start_destroy { id: asset.into() })
				.get_dispatch_info()
				.weight;

		// This is the db write
		call_weight.saturating_add(<Runtime as frame_system::Config>::DbWeight::get().writes(1))
	}
}

pub struct LocalAssetIdCreator;
impl pallet_asset_manager::LocalAssetIdCreator<Runtime> for LocalAssetIdCreator {
	fn create_asset_id_from_metadata(_local_asset_counter: u128) -> crate::AssetId {
		// We don't need to create local asset.
		unimplemented!()
	}
}

impl pallet_asset_manager::Config for Runtime {
	type AssetId = crate::AssetId;
	type AssetRegistrar = AssetRegistrar;
	type AssetRegistrarMetadata = xcm_configs::AssetRegistrarMetadata;
	type Balance = Balance;
	type Currency = Balances;
	type ForeignAssetModifierOrigin = Root;
	type ForeignAssetType = xcm_configs::AssetType;
	type LocalAssetDeposit = ConstU128<0>;
	type LocalAssetIdCreator = LocalAssetIdCreator;
	type LocalAssetModifierOrigin = Root;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_asset_manager::weights::SubstrateWeight<Runtime>;
}
