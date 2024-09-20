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
#[allow(unused_imports)]
use crate::*;
// polkadot-sdk
#[allow(unused_imports)]
use frame_support::{migration, storage::unhashed};

pub struct CustomOnRuntimeUpgrade;
impl frame_support::traits::OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		log::info!("pre");

		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		log::info!("post");

		Ok(())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		migrate()
	}
}

fn migrate() -> frame_support::weights::Weight {
	let _ = migration::clear_storage_prefix(
		b"BridgeKusamaGrandpa",
		b"ImportedHeaders",
		&[],
		Some(100),
		None,
	);

	// dawinia
	use darwinia_staking::CacheState;
	if let Some(s) = migration::get_storage_value::<(CacheState, CacheState, CacheState)>(
		b"DarwinaStaking",
		b"ExposureCacheStates",
		&[],
	) {
		migration::put_storage_value(b"DarwinaStaking", b"CacheStates", &[], s);
	}

	if let Ok(dao) =
		array_bytes::hex_n_into::<_, AccountId, 20>("0x08837De0Ae21C270383D9F2de4DB03c7b1314632")
	{
		let _ = <pallet_assets::Pallet<Runtime>>::transfer_ownership(
			RuntimeOrigin::signed(ROOT),
			codec::Compact(AssetIds::Kton as AssetId),
			dao,
		);

		if let Ok(deposit) = array_bytes::hex_n_into::<_, AccountId, 20>(
			"0x46275d29113f065c2aac262f34C7a3d8a8B7377D",
		) {
			let _ = <pallet_assets::Pallet<Runtime>>::set_team(
				RuntimeOrigin::signed(dao),
				codec::Compact(AssetIds::Kton as AssetId),
				deposit,
				deposit,
				dao,
			);

			<darwinia_deposit::DepositContract<Runtime>>::put(deposit);
		}
	}
	if let Ok(who) =
		array_bytes::hex_n_into::<_, AccountId, 20>("0xa4fFAC7A5Da311D724eD47393848f694Baee7930")
	{
		<darwinia_staking::RingStakingContract<Runtime>>::put(who);
	}

	// frame_support::weights::Weight::zero()
	<Runtime as frame_system::Config>::DbWeight::get().reads_writes(7, 107)
}
