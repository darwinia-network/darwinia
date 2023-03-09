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
#[allow(unused_imports)]
use crate::*;
// substrate
#[allow(unused_imports)]
use frame_support::{log, migration};

const O_STAKING: &[u8] = b"Staking";
const O_RING_POOL: &[u8] = b"RingPool";
const O_KTON_POOL: &[u8] = b"KtonPool";
const O_ELAPSED_TIME: &[u8] = b"ElapsedTime";

pub struct CustomOnRuntimeUpgrade;
impl frame_support::traits::OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
		Ok(())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		migrate()
	}
}

fn migrate() -> frame_support::weights::Weight {
	<darwinia_staking::RingPool<Runtime>>::mutate(|v| {
		if let Some(o) = migration::take_storage_value::<Balance>(O_STAKING, O_RING_POOL, &[]) {
			log::info!("current     `ring_pool({v})`");
			log::info!("un-migrated `ring_pool({o})`");

			*v += o;

			log::info!("fixed       `ring_pool({v})`");
		}
	});
	<darwinia_staking::KtonPool<Runtime>>::mutate(|v| {
		if let Some(o) = migration::take_storage_value::<Balance>(O_STAKING, O_KTON_POOL, &[]) {
			log::info!("current     `kton_pool({v})`");
			log::info!("un-migrated `kton_pool({o})`");

			*v += o;

			log::info!("fixed       `kton_pool({v})`");
		}
	});
	<darwinia_staking::ElapsedTime<Runtime>>::mutate(|v| {
		if let Some(o) = migration::take_storage_value::<Moment>(O_STAKING, O_ELAPSED_TIME, &[]) {
			log::info!("current     `elapsed_time({v})`");
			log::info!("un-migrated `elapsed_time({o})`");
			log::info!("genesis     `elapsed_time(11_516_352_020)`");

			*v += o;
			*v -= 11_516_352_020;

			log::info!("fixed       `elapsed_time({v})`");
		}
	});

	// frame_support::weights::Weight::zero()
	// RuntimeBlockWeights::get().max_block
	<Runtime as frame_system::Config>::DbWeight::get().reads_writes(3, 3)
}
