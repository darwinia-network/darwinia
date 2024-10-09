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
	// darwinia
	use darwinia_staking::CacheState;
	if let Some(s) = migration::get_storage_value::<(CacheState, CacheState, CacheState)>(
		b"DarwiniaStaking",
		b"ExposureCacheStates",
		&[],
	) {
		migration::put_storage_value(b"DarwiniaStaking", b"CacheStates", &[], s);
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

		log::info!("successfully transfer ownership of KTON to KTON DAO");
	}
	if let Ok(who) =
		array_bytes::hex_n_into::<_, AccountId, 20>("0xa4fFAC7A5Da311D724eD47393848f694Baee7930")
	{
		<darwinia_staking::RingStakingContract<Runtime>>::put(who);

		log::info!("successfully set RING staking contract");
	}

	<darwinia_staking::MigrationStartPoint<Runtime>>::put(darwinia_staking::now::<Runtime>());

	if let Some(k) = migration::take_storage_value::<AccountId>(
		b"DarwiniaStaking",
		b"KtonRewardDistributionContract",
		&[],
	) {
		<darwinia_staking::KtonStakingContract<Runtime>>::put(k);

		log::info!("successfully set KTON staking contract");
	}

	if let Ok(k) = array_bytes::hex2bytes("0x1da53b775b270400e7e61ed5cbc5a146ab1160471b1418779239ba8e2b847e42d53de13b56da115d3342f0588bc3614108837de0ae21c270383d9f2de4db03c7b1314632314d8c74970d627c9b4f4c42e06688a9f7a2866905a810c4b1a49b8cb0dca3f1bc953905609869b6e9d4fb794cd36c5f") {
		System::kill_storage(RuntimeOrigin::root(), vec![k]);
	}

	<Runtime as frame_system::Config>::DbWeight::get().reads_writes(7, 10)
}
