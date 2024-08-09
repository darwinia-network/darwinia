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
	let mut n = 100;

	n += migration_helper::PalletCleaner {
		name: b"EthereumXcm",
		values: &[b"Nonce", b"EthereumXcmSuspended"],
		maps: &[],
	}
	.remove_storage_values();

	const KTON_DAO_VAULT_ADDR: &str = "0xf1b4f3D438eE2B363C5ba1641A498709ff5780bA";

	#[cfg(feature = "try-runtime")]
	assert!(
		array_bytes::hex_n_into::<_, _, 20>(KTON_DAO_VAULT_ADDR).is_ok()
	);

	if let Some(w) =
		array_bytes::hex_n_into::<_, _, 20>(KTON_DAO_VAULT_ADDR)
	{
		<darwinia_staking::KtonRewardDistributionContract<Runtime>>::put(w);
		darwinia_staking::migration::migrate_staking_reward_distribution_contract::<Runtime>(w);
	}

	// frame_support::weights::Weight::zero()
	<Runtime as frame_system::Config>::DbWeight::get().reads_writes(10, n + 10)
}
