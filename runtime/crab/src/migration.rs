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
	// substrate
	use codec::Encode;
	use frame_support::{StorageHasher, Twox64Concat};
	use sp_core::U256;

	let number = System::block_number();
	let old_block_hash_count = 2400;
	let new_block_hash_count = 256;
	let old_to_remove = number.saturating_sub(old_block_hash_count).saturating_sub(1);
	let new_to_remove_before_finalize =
		number.saturating_sub(new_block_hash_count).saturating_sub(1).saturating_sub(1);

	// keep genesis hash
	if old_to_remove != 0 {
		for to_remove in old_to_remove..=new_to_remove_before_finalize {
			<frame_system::BlockHash<Runtime>>::remove(to_remove);

			// StorageItem link: https://github.com/paritytech/frontier/blob/polkadot-v0.9.38/frame/ethereum/src/lib.rs#L338
			// Since this storage item is private at `polkadot-v0.9.38` branch, we have to migrate it manually. There https://github.com/paritytech/frontier/pull/1034 change the visibility of this item to public.
			// But I think this is not a complicated one to review, so let's do it.
			let _ = migration::clear_storage_prefix(
				b"Ethereum",
				b"BlockHash",
				&Twox64Concat::hash(&U256::from(to_remove).encode()),
				None,
				None,
			);
		}
	}

	<Runtime as frame_system::Config>::DbWeight::get().reads_writes(0, 1)
		* 2 * (new_to_remove_before_finalize - old_to_remove + 1) as u64
}
