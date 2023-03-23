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
use frame_support::{log, migration, storage::unhashed};

// Crab2
// AccountMigration::Vestings 0x1fb3231abc71c5a12c573bc57e9d12d1c5b10e12d951c676432baac73459815c
//
// Crab
// Vesting::Vesting           0x5f27b51b5ec208ee9cb25b55d87282435f27b51b5ec208ee9cb25b55d8728243
// Expired.
// 0x5f27b51b5ec208ee9cb25b55d87282435f27b51b5ec208ee9cb25b55d87282433f2770dc0c68232d160e98225bc421c9608c62275934b164899ca6270c4b89c5d84b2390d4316fda980cd1b3acfad525
// {
// 	locked: 1,000,000,000
// 	perBlock: 500,000,000
// 	startingBlock: 6,740,780
// }
// 100,000,000,000 - (CRAB_LAST_FINALIZE_HEIGHT - 8,421,033) * 1
// 0x5f27b51b5ec208ee9cb25b55d87282435f27b51b5ec208ee9cb25b55d8728243c06c2bc8f107407659c073c78f8a3b94360a95cd317b649c62ea53444ca50e9403031cec3fb9ac3650d4b72b88260f0c
// {
// 	locked: 100,000,000,000
// 	perBlock: 1
// 	startingBlock: 8,421,033
// }
// Expired.
// 0x5f27b51b5ec208ee9cb25b55d87282435f27b51b5ec208ee9cb25b55d8728243d52763ae3a2afb1c9c77d6426427422f72f9aa4c12882beefc97bba96617ed811690f68bb917e4b77d2dcc6d717a4422
// {
// 	locked: 1,000,000,000
// 	perBlock: 1,000,000,000
// 	startingBlock: 6,829,787
// }
const BROKEN_STORAGES: &[&str] = &[
	"0x1fb3231abc71c5a12c573bc57e9d12d1c5b10e12d951c676432baac73459815c3f2770dc0c68232d160e98225bc421c9608c62275934b164899ca6270c4b89c5d84b2390d4316fda980cd1b3acfad525",
	"0x1fb3231abc71c5a12c573bc57e9d12d1c5b10e12d951c676432baac73459815cc06c2bc8f107407659c073c78f8a3b94360a95cd317b649c62ea53444ca50e9403031cec3fb9ac3650d4b72b88260f0c",
	"0x1fb3231abc71c5a12c573bc57e9d12d1c5b10e12d951c676432baac73459815cd52763ae3a2afb1c9c77d6426427422f72f9aa4c12882beefc97bba96617ed811690f68bb917e4b77d2dcc6d717a4422",
];
const CRAB_LAST_FINALIZE_HEIGHT: BlockNumber = 14_741_053;
const DECIMAL_OFFSET: Balance = 1_000_000_000;

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

#[allow(clippy::identity_op)]
fn migrate() -> frame_support::weights::Weight {
	#[derive(codec::Encode, codec::Decode, frame_support::RuntimeDebug)]
	struct VestingInfo {
		locked: Balance,
		per_block: Balance,
		starting_block: BlockNumber,
	}

	if let Ok(k) = array_bytes::hex2bytes(BROKEN_STORAGES[0]) {
		// If `is_some` which means it hasn't been migrated yet.
		// But actually, without this correction, the account migration will fail.
		if unhashed::get::<VestingInfo>(&k).is_some() {
			log::info!("purge `storage({})`", BROKEN_STORAGES[0]);
			unhashed::kill(&k);
		}
	}
	if let Ok(k) = array_bytes::hex2bytes(BROKEN_STORAGES[1]) {
		// If `is_some` which means it hasn't been migrated yet.
		// But actually, without this correction, the account migration will fail.
		if unhashed::get::<VestingInfo>(&k).is_some() {
			let v = vec![VestingInfo {
				locked: (100_000_000_000_u128
					- (CRAB_LAST_FINALIZE_HEIGHT - 8_421_033) as Balance * 1_u128)
					* DECIMAL_OFFSET,
				// Crab2's block time is twice longer than Crab.
				per_block: 1 * DECIMAL_OFFSET * 2,
				starting_block: 0,
			}];

			log::info!("correct `storage({})` to `{v:?}`", BROKEN_STORAGES[1]);
			unhashed::put::<Vec<VestingInfo>>(&k, &v);
		}
	}
	if let Ok(k) = array_bytes::hex2bytes(BROKEN_STORAGES[2]) {
		// If `is_some` which means it hasn't been migrated yet.
		// But actually, without this correction, the account migration will fail.
		if unhashed::get::<VestingInfo>(&k).is_some() {
			log::info!("purge `storage({})`", BROKEN_STORAGES[2]);
			unhashed::kill(&k);
		}
	}

	// frame_support::weights::Weight::zero()
	// RuntimeBlockWeights::get().max_block
	<Runtime as frame_system::Config>::DbWeight::get().reads_writes(3, 3)
}
