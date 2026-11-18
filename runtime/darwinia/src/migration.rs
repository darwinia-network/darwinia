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
use codec::DecodeAll;
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
}

fn kton_details() -> Result<storage::AssetDetails, &'static str> {
	let key = storage::Asset::hashed_key_for(AssetIds::Kton as AssetId);
	let raw = frame_support::storage::unhashed::get_raw(&key).ok_or("KTON asset missing")?;
	// Re-encoding a partially decoded record would drop any unknown trailing fields.
	storage::AssetDetails::decode_all(&mut &raw[..]).map_err(|_| "Unexpected KTON asset layout")
}

pub struct CustomOnRuntimeUpgrade;
impl OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		snapshot()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		let asset = kton_details()?;
		frame_support::ensure!(
			asset.status == storage::AssetStatus::Frozen,
			"KTON must be frozen after the upgrade"
		);
		frame_support::ensure!(
			snapshot()? == state,
			"KTON freeze changed holdings, issuance, permissions, metadata, approvals, or collator count"
		);
		Ok(())
	}

	fn on_runtime_upgrade() -> Weight {
		migrate()
	}
}

/// Freeze existing KTON holdings in place for the Ethereum migration snapshot.
fn migrate() -> Weight {
	let id = AssetIds::Kton as AssetId;
	let db = <Runtime as frame_system::Config>::DbWeight::get();
	let mut asset = match kton_details() {
		Ok(asset) => asset,
		Err(reason) => {
			log::error!("KTON freeze: {reason}");
			return db.reads(1);
		},
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
fn kton_prefix(item: &[u8]) -> Vec<u8> {
	use frame_support::{storage::storage_prefix, traits::PalletInfoAccess, Hashable};
	let mut prefix = storage_prefix(Assets::name().as_bytes(), item).to_vec();
	prefix.extend((AssetIds::Kton as AssetId).blake2_128_concat());
	prefix
}

#[cfg(feature = "try-runtime")]
fn entries_digest(item: &str, expected_count: u32) -> Result<[u8; 32], sp_runtime::DispatchError> {
	let prefix = kton_prefix(item.as_bytes());
	// Compare exact keys and values, including records a typed iterator might skip.
	let entries =
		frame_support::storage::PrefixIterator::<_>::new(prefix.clone(), prefix, |k, v| {
			Ok((k.to_vec(), v.to_vec()))
		})
		.collect::<Vec<_>>();
	if entries.len() != expected_count as usize {
		log::error!(
			"KTON {item}: expected {expected_count} storage entries, found {}",
			entries.len()
		);
		return Err("KTON storage count mismatch".into());
	}
	Ok(entries.using_encoded(sp_core::hashing::blake2_256))
}

#[cfg(feature = "try-runtime")]
fn snapshot() -> Result<Vec<u8>, sp_runtime::DispatchError> {
	let mut asset = kton_details()?;
	frame_support::ensure!(
		matches!(asset.status, storage::AssetStatus::Live | storage::AssetStatus::Frozen),
		"Cannot freeze a destroying KTON asset"
	);
	// Normalize the single field this upgrade is allowed to change.
	asset.status = storage::AssetStatus::Frozen;
	let accounts = entries_digest("Account", asset.accounts)?;
	let approvals = entries_digest("Approvals", asset.approvals)?;
	Ok((
		asset,
		accounts,
		approvals,
		frame_support::storage::unhashed::get_raw(&kton_prefix(b"Metadata")),
		darwinia_staking::CollatorCount::<Runtime>::get(),
	)
		.encode())
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::{assert_noop, assert_ok, traits::fungibles::Inspect, Hashable};
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
			let mut legacy_key =
				frame_support::storage::storage_prefix(b"AccountMigration", b"KtonAccounts")
					.to_vec();
			legacy_key.extend([0xABu8; 32].blake2_128_concat());
			// Legacy AssetAccount with a balance, a liquid status and a sufficient reference.
			let legacy_record = (42u128, false, 1u8, ()).encode();
			sp_io::storage::set(&legacy_key, &legacy_record);
			let mut expected = storage::Asset::get(id).unwrap();
			expected.status = storage::AssetStatus::Frozen;
			let db = <Runtime as frame_system::Config>::DbWeight::get();

			assert_eq!(CustomOnRuntimeUpgrade::on_runtime_upgrade(), db.reads_writes(1, 1));
			assert_eq!(storage::Asset::get(id), Some(expected.clone()));
			assert_eq!(CustomOnRuntimeUpgrade::on_runtime_upgrade(), db.reads(1));
			assert_eq!(storage::Asset::get(id), Some(expected));
			assert_eq!(Assets::balance(id, ROOT), 100);
			assert_eq!(Assets::total_issuance(id), 100);
			assert_eq!(darwinia_staking::CollatorCount::<Runtime>::get(), 17);
			assert_eq!(&sp_io::storage::get(&legacy_key).unwrap()[..], legacy_record.as_slice());
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
			let recipient = H160::repeat_byte(0xBB).into();
			let delegate = H160::repeat_byte(0xCC).into();
			assert_ok!(Assets::approve_transfer(
				RuntimeOrigin::signed(ROOT),
				id.into(),
				delegate,
				25
			));
			CustomOnRuntimeUpgrade::on_runtime_upgrade();
			let frozen =
				sp_runtime::DispatchError::from(pallet_assets::Error::<Runtime>::AssetNotLive);
			assert_noop!(
				Assets::transfer(RuntimeOrigin::signed(ROOT), id.into(), recipient, 10),
				frozen
			);
			assert_noop!(
				Assets::mint(RuntimeOrigin::signed(ROOT), id.into(), recipient, 10),
				frozen
			);
			assert_noop!(Assets::burn(RuntimeOrigin::signed(ROOT), id.into(), ROOT, 10), frozen);
			assert_noop!(
				Assets::force_transfer(RuntimeOrigin::signed(ROOT), id.into(), ROOT, recipient, 10),
				frozen
			);
			assert_noop!(
				Assets::transfer_approved(
					RuntimeOrigin::signed(delegate),
					id.into(),
					ROOT,
					recipient,
					10
				),
				frozen
			);
			assert_eq!(Assets::balance(id, ROOT), 100);
			assert_eq!(Assets::total_issuance(id), 100);
		});
	}

	#[test]
	fn unexpected_asset_layout_is_left_intact() {
		externalities().execute_with(|| {
			let key = storage::Asset::hashed_key_for(AssetIds::Kton as AssetId);
			let mut extended = sp_io::storage::get(&key).unwrap().to_vec();
			extended.push(1);
			for raw in [extended, vec![0]] {
				sp_io::storage::set(&key, &raw);
				#[cfg(feature = "try-runtime")]
				assert!(CustomOnRuntimeUpgrade::pre_upgrade().is_err());
				let db = <Runtime as frame_system::Config>::DbWeight::get();
				assert_eq!(CustomOnRuntimeUpgrade::on_runtime_upgrade(), db.reads(1));
				assert_eq!(sp_io::storage::get(&key).unwrap().as_ref(), raw.as_slice());
			}
		});
	}

	#[cfg(feature = "try-runtime")]
	#[test]
	fn try_runtime_checks_preserve_the_snapshot() {
		externalities().execute_with(|| {
			for _ in 0..2 {
				let before = CustomOnRuntimeUpgrade::pre_upgrade().unwrap();
				CustomOnRuntimeUpgrade::on_runtime_upgrade();
				assert_ok!(CustomOnRuntimeUpgrade::post_upgrade(before));
			}
			let before = CustomOnRuntimeUpgrade::pre_upgrade().unwrap();
			storage::Asset::mutate(AssetIds::Kton as AssetId, |asset| {
				asset.as_mut().unwrap().supply += 1
			});
			assert!(CustomOnRuntimeUpgrade::post_upgrade(before).is_err());
		});
	}

	#[cfg(feature = "try-runtime")]
	#[test]
	fn snapshot_detects_redistribution_with_constant_issuance() {
		externalities().execute_with(|| {
			let id = AssetIds::Kton as AssetId;
			let holder = H160::repeat_byte(0xBB).into();
			assert_ok!(Assets::transfer(RuntimeOrigin::signed(ROOT), id.into(), holder, 25));
			let before = CustomOnRuntimeUpgrade::pre_upgrade().unwrap();
			assert_ok!(Assets::transfer(RuntimeOrigin::signed(ROOT), id.into(), holder, 10));
			let after = CustomOnRuntimeUpgrade::pre_upgrade().unwrap();
			CustomOnRuntimeUpgrade::on_runtime_upgrade();
			assert_eq!(Assets::total_issuance(id), 100);
			assert!(CustomOnRuntimeUpgrade::post_upgrade(before).is_err());
			assert_ok!(CustomOnRuntimeUpgrade::post_upgrade(after));
		});
	}

	#[cfg(feature = "try-runtime")]
	#[test]
	fn snapshot_detects_changes_to_an_existing_allowance() {
		externalities().execute_with(|| {
			let id = AssetIds::Kton as AssetId;
			let delegate = H160::repeat_byte(0xBB).into();
			assert_ok!(Assets::approve_transfer(
				RuntimeOrigin::signed(ROOT),
				id.into(),
				delegate,
				25
			));
			let before = CustomOnRuntimeUpgrade::pre_upgrade().unwrap();
			assert_ok!(Assets::approve_transfer(
				RuntimeOrigin::signed(ROOT),
				id.into(),
				delegate,
				10
			));
			let after = CustomOnRuntimeUpgrade::pre_upgrade().unwrap();
			CustomOnRuntimeUpgrade::on_runtime_upgrade();
			assert!(CustomOnRuntimeUpgrade::post_upgrade(before).is_err());
			assert_ok!(CustomOnRuntimeUpgrade::post_upgrade(after));
		});
	}

	#[cfg(feature = "try-runtime")]
	#[test]
	fn snapshot_rejects_missing_account_or_approval_rows() {
		for item in [b"Account".as_slice(), b"Approvals"] {
			externalities().execute_with(|| {
				let id = AssetIds::Kton as AssetId;
				let delegate = H160::repeat_byte(0xBB).into();
				assert_ok!(Assets::approve_transfer(
					RuntimeOrigin::signed(ROOT),
					id.into(),
					delegate,
					25
				));
				assert_ok!(CustomOnRuntimeUpgrade::pre_upgrade());
				let prefix = kton_prefix(item);
				let key = sp_io::storage::next_key(&prefix).unwrap();
				assert!(key.starts_with(&prefix));
				sp_io::storage::clear(&key);
				assert!(CustomOnRuntimeUpgrade::pre_upgrade().is_err());
			});
		}
	}

	#[cfg(feature = "try-runtime")]
	#[test]
	fn pre_upgrade_rejects_missing_or_destroying_kton() {
		externalities().execute_with(|| {
			let id = AssetIds::Kton as AssetId;
			storage::Asset::mutate(id, |asset| {
				asset.as_mut().unwrap().status = storage::AssetStatus::Destroying
			});
			assert!(CustomOnRuntimeUpgrade::pre_upgrade().is_err());
			CustomOnRuntimeUpgrade::on_runtime_upgrade();
			assert_eq!(storage::Asset::get(id).unwrap().status, storage::AssetStatus::Destroying);
			storage::Asset::remove(id);
			assert!(CustomOnRuntimeUpgrade::pre_upgrade().is_err());
			CustomOnRuntimeUpgrade::on_runtime_upgrade();
			assert!(!storage::Asset::contains_key(id));
		});
	}
}
