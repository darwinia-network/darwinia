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
// substrate
use frame_support::pallet_prelude::*;
use frame_support::dispatch::GetDispatchInfo;
use sp_runtime::traits::Hash;
use sp_std::prelude::*;

#[derive(Clone, Default, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub struct AssetRegistrarMetadata {
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
	pub decimals: u8,
	pub is_frozen: bool,
}

// Our AssetType. For now we only handle Xcm Assets
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
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
// We simply hash the AssetType and take the lowest 128 bits
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

// We instruct how to register the Assets
// In this case, we tell it to Create an Asset in pallet-assets
pub struct AssetRegistrar;
use frame_support::{pallet_prelude::DispatchResult, transactional};

impl pallet_asset_manager::AssetRegistrar<Runtime> for AssetRegistrar {
	#[transactional]
	fn create_foreign_asset(
		asset: crate::AssetId,
		min_balance: Balance,
		metadata: AssetRegistrarMetadata,
		is_sufficient: bool,
	) -> DispatchResult {
		Assets::force_create(
			RuntimeOrigin::root(),
			asset.into(),
			super::super::AssetManager::account_id(),
			is_sufficient,
			min_balance,
		)?;

		// TODO uncomment when we feel comfortable
		/*
		// The asset has been created. Let's put the revert code in the precompile address
		let precompile_address = Runtime::asset_id_to_account(ASSET_PRECOMPILE_ADDRESS_PREFIX, asset);
		pallet_evm::AccountCodes::<Runtime>::insert(
			precompile_address,
			vec![0x60, 0x00, 0x60, 0x00, 0xfd],
		);*/

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

		// TODO Check this
		/* 
		// We remove the EVM revert code
		// This does not panick even if there is no code in the address
		// let precompile_address: sp_core::H160 =
		// 	Runtime::asset_id_to_account(FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX, asset).into();
		// pallet_evm::AccountCodes::<Runtime>::remove(precompile_address);*/
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
		let call_weight = RuntimeCall::Assets(
			pallet_assets::Call::<Runtime>::start_destroy {
				id: asset.into(),
			},
		)
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
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = crate::AssetId;
	type AssetRegistrarMetadata = AssetRegistrarMetadata;
	type ForeignAssetType = AssetType;
	type AssetRegistrar = AssetRegistrar;
	type ForeignAssetModifierOrigin = Root;
	type LocalAssetModifierOrigin = Root;
	type LocalAssetIdCreator = LocalAssetIdCreator;
	type Currency = Balances;
	type LocalAssetDeposit = ConstU128<0>;
	type WeightInfo = pallet_asset_manager::weights::SubstrateWeight<Runtime>;
}