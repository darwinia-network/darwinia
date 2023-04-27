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

const OLD_BLOCK_HASH_COUNT: BlockNumber = 2400;
const NEW_BLOCK_HASH_COUNT: BlockNumber = 256;

pub struct CustomOnRuntimeUpgrade;
impl frame_support::traits::OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
		assert_eq!(<frame_system::BlockHash<Runtime>>::iter().count() as BlockNumber, NEW_BLOCK_HASH_COUNT);
		assert_eq!(
			migration::storage_iter::<()>(b"Ethereum", b"BlockHash").count() as BlockNumber,
			NEW_BLOCK_HASH_COUNT
		);

		Ok(())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		migrate()
	}
}

fn migrate() -> frame_support::weights::Weight {
	// crates.io
	use codec::Encode;
	// substrate
	use frame_support::{StorageHasher, Twox64Concat};
	use sp_core::U256;

	let now = System::block_number();
	// now - 2400, now - 2399, .. now
	// ->
	// now - 256, now - 255, .. now
	// =
	// purge(now - 2400 ..= now - 255)
	let start = now.saturating_sub(OLD_BLOCK_HASH_COUNT);
	let end = now.saturating_sub(NEW_BLOCK_HASH_COUNT);

	// keep genesis hash
	if start != 0 {
		for n in start..end {
			<frame_system::BlockHash<Runtime>>::remove(n);

			// Storage item: https://github.com/paritytech/frontier/blob/polkadot-v0.9.38/frame/ethereum/src/lib.rs#L338
			// Since this storage item is private at `polkadot-v0.9.38` branch, we have to migrate
			// it manually. https://github.com/paritytech/frontier/pull/1034 changes the visibility of this item to public.
			// This is not a complicated one to review, so let's do it.
			let _ = migration::take_storage_value::<()>(
				b"Ethereum",
				b"BlockHash",
				&Twox64Concat::hash(&U256::from(n).encode()),
			);
		}
	}

	<Runtime as frame_system::Config>::DbWeight::get()
		.reads_writes(0, (2 * end.saturating_sub(start)) as _)
}
