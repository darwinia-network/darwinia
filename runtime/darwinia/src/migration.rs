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

use crate::*;
use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};

// The SDK keeps asset details and status private. This alias mirrors the SCALE
// layout in pallet-assets 40.0.0, pinned by Cargo.lock. Only status is changed.
mod storage {
	use super::*;
	use frame_support::{pallet_prelude::*, storage_alias};

	#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
	pub enum AssetStatus {
		#[codec(index = 0)]
		Live,
		#[codec(index = 1)]
		Frozen,
		#[codec(index = 2)]
		Destroying,
	}

	#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
	pub struct AssetDetails {
		pub owner: AccountId,
		pub issuer: AccountId,
		pub admin: AccountId,
		pub freezer: AccountId,
		pub supply: Balance,
		pub deposit: Balance,
		pub min_balance: Balance,
		pub is_sufficient: bool,
		pub accounts: u32,
		pub sufficients: u32,
		pub approvals: u32,
		pub status: AssetStatus,
	}

	#[storage_alias]
	pub type Asset = StorageMap<Assets, Blake2_128Concat, AssetId, AssetDetails>;

	#[cfg(feature = "try-runtime")]
	#[storage_alias]
	pub type Account = StorageDoubleMap<
		Assets,
		Blake2_128Concat,
		AssetId,
		Blake2_128Concat,
		AccountId,
		pallet_assets::AssetAccount<Balance, Balance, (), AccountId>,
	>;

	#[cfg(feature = "try-runtime")]
	#[storage_alias]
	pub type Approvals = StorageNMap<
		Assets,
		(
			NMapKey<Blake2_128Concat, AssetId>,
			NMapKey<Blake2_128Concat, AccountId>,
			NMapKey<Blake2_128Concat, AccountId>,
		),
		pallet_assets::Approval<Balance, Balance>,
	>;

	#[cfg(feature = "try-runtime")]
	#[storage_alias]
	pub type Metadata = StorageMap<
		Assets,
		Blake2_128Concat,
		AssetId,
		pallet_assets::AssetMetadata<
			Balance,
			BoundedVec<u8, <Runtime as pallet_assets::Config>::StringLimit>,
		>,
		ValueQuery,
	>;
}

/// Freeze existing KTON holdings without moving balances, burning supply, or unstaking users.
pub struct FreezeKton;
impl OnRuntimeUpgrade for FreezeKton {
	fn on_runtime_upgrade() -> Weight {
		let id = AssetIds::Kton as AssetId;
		let db = <Runtime as frame_system::Config>::DbWeight::get();
		let Some(mut asset) = storage::Asset::get(id) else {
			log::error!("KTON freeze: asset {id} is missing; pre-upgrade checks must pass");
			return db.reads(1);
		};

		match asset.status {
			storage::AssetStatus::Live => {
				asset.status = storage::AssetStatus::Frozen;
				storage::Asset::insert(id, asset);
				log::info!("KTON asset {id} frozen for the Ethereum migration snapshot");
				db.reads_writes(1, 1)
			},
			storage::AssetStatus::Frozen => db.reads(1),
			storage::AssetStatus::Destroying => {
				// Never resurrect an asset whose destruction has already started.
				log::error!("KTON freeze: asset {id} is destroying; pre-upgrade checks must pass");
				db.reads(1)
			},
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		Self::snapshot()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		let asset = storage::Asset::get(AssetIds::Kton as AssetId).ok_or("KTON asset missing")?;
		frame_support::ensure!(
			asset.status == storage::AssetStatus::Frozen,
			"KTON must be frozen after the upgrade"
		);
		frame_support::ensure!(
			Self::snapshot()? == state,
			"KTON freeze changed holdings, issuance, permissions, metadata, approvals, or collator count"
		);
		Ok(())
	}
}

#[cfg(feature = "try-runtime")]
impl FreezeKton {
	fn snapshot() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		let id = AssetIds::Kton as AssetId;
		let mut asset = storage::Asset::get(id).ok_or("KTON asset missing")?;
		frame_support::ensure!(
			matches!(asset.status, storage::AssetStatus::Live | storage::AssetStatus::Frozen),
			"Cannot freeze a destroying KTON asset"
		);
		// Normalize the single field this upgrade is allowed to change.
		asset.status = storage::AssetStatus::Frozen;
		let accounts = storage::Account::iter_prefix(id).collect::<Vec<_>>();
		let approvals = storage::Approvals::iter_prefix((id,)).collect::<Vec<_>>();
		Ok((
			asset,
			sp_core::hashing::blake2_256(&accounts.encode()),
			sp_core::hashing::blake2_256(&approvals.encode()),
			storage::Metadata::get(id),
			darwinia_staking::CollatorCount::<Runtime>::get(),
		)
			.encode())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::{assert_ok, traits::fungibles::Inspect};
	use sp_runtime::BuildStorage;

	fn externalities() -> sp_io::TestExternalities {
		let mut state = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();
		pallet_assets::GenesisConfig::<Runtime> {
			assets: vec![(AssetIds::Kton as AssetId, ROOT, true, 1)],
			metadata: vec![(AssetIds::Kton as AssetId, b"KTON".to_vec(), b"KTON".to_vec(), 18)],
			accounts: vec![(AssetIds::Kton as AssetId, ROOT, 100)],
			next_asset_id: None,
		}
		.assimilate_storage(&mut state)
		.unwrap();
		state.into()
	}

	#[test]
	fn freeze_changes_only_status_and_is_idempotent() {
		externalities().execute_with(|| {
			System::set_block_number(1);
			let id = AssetIds::Kton as AssetId;
			let delegate = H160::repeat_byte(0xBB).into();
			assert_ok!(Assets::approve_transfer(
				RuntimeOrigin::signed(ROOT),
				id.into(),
				delegate,
				25
			));
			// Unrelated governance settings and retained legacy data must not be changed.
			darwinia_staking::CollatorCount::<Runtime>::put(17);
			let legacy_key =
				frame_support::storage::storage_prefix(b"AccountMigration", b"KtonAccounts");
			sp_io::storage::set(&legacy_key, b"retained legacy records");
			let mut expected = storage::Asset::get(id).unwrap();
			expected.status = storage::AssetStatus::Frozen;
			let db = <Runtime as frame_system::Config>::DbWeight::get();

			assert_eq!(FreezeKton::on_runtime_upgrade(), db.reads_writes(1, 1));
			assert_eq!(storage::Asset::get(id), Some(expected.clone()));
			assert_eq!(FreezeKton::on_runtime_upgrade(), db.reads(1));
			assert_eq!(storage::Asset::get(id), Some(expected));
			assert_eq!(Assets::balance(id, ROOT), 100);
			assert_eq!(Assets::total_issuance(id), 100);
			assert_eq!(darwinia_staking::CollatorCount::<Runtime>::get(), 17);
			assert_eq!(&sp_io::storage::get(&legacy_key).unwrap()[..], b"retained legacy records");
			// Check the mirrored storage against the SDK's actual codec.
			let raw = sp_io::storage::get(&storage::Asset::hashed_key_for(id)).unwrap();
			let sdk =
				pallet_assets::AssetDetails::<Balance, AccountId, Balance>::decode(&mut &raw[..])
					.unwrap();
			assert_eq!(sdk.encode().as_slice(), &raw[..]);
		});
	}

	#[test]
	fn frozen_asset_rejects_direct_internal_balance_changes() {
		externalities().execute_with(|| {
			let id = AssetIds::Kton as AssetId;
			FreezeKton::on_runtime_upgrade();
			let recipient = H160::repeat_byte(0xBB).into();
			assert!(
				Assets::transfer(RuntimeOrigin::signed(ROOT), id.into(), recipient, 10).is_err()
			);
			assert!(Assets::mint(RuntimeOrigin::signed(ROOT), id.into(), recipient, 10).is_err());
			assert!(Assets::burn(RuntimeOrigin::signed(ROOT), id.into(), ROOT, 10).is_err());
			assert!(Assets::force_transfer(
				RuntimeOrigin::signed(ROOT),
				id.into(),
				ROOT,
				recipient,
				10
			)
			.is_err());
			assert_eq!(Assets::balance(id, ROOT), 100);
			assert_eq!(Assets::total_issuance(id), 100);
		});
	}

	#[cfg(feature = "try-runtime")]
	#[test]
	fn try_runtime_checks_preserve_the_snapshot() {
		externalities().execute_with(|| {
			for _ in 0..2 {
				let before = FreezeKton::pre_upgrade().unwrap();
				FreezeKton::on_runtime_upgrade();
				assert_ok!(FreezeKton::post_upgrade(before));
			}
			let before = FreezeKton::pre_upgrade().unwrap();
			storage::Asset::mutate(AssetIds::Kton as AssetId, |asset| {
				asset.as_mut().unwrap().supply += 1
			});
			assert!(FreezeKton::post_upgrade(before).is_err());
		});
	}

	#[cfg(feature = "try-runtime")]
	#[test]
	fn pre_upgrade_rejects_missing_or_destroying_kton() {
		externalities().execute_with(|| {
			let id = AssetIds::Kton as AssetId;
			storage::Asset::mutate(id, |asset| {
				asset.as_mut().unwrap().status = storage::AssetStatus::Destroying
			});
			assert!(FreezeKton::pre_upgrade().is_err());
			FreezeKton::on_runtime_upgrade();
			assert_eq!(storage::Asset::get(id).unwrap().status, storage::AssetStatus::Destroying);
			storage::Asset::remove(id);
			assert!(FreezeKton::pre_upgrade().is_err());
			FreezeKton::on_runtime_upgrade();
			assert!(!storage::Asset::contains_key(id));
		});
	}
}
