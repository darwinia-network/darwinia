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
// substrate
#[allow(unused_imports)]
use frame_support::{migration, storage::unhashed};

pub struct CustomOnRuntimeUpgrade;
impl frame_support::traits::OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		assert!(migration::have_storage_value(b"EcdsaAuthority", b"Authorities", &[]));

		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		assert!(!migration::have_storage_value(b"EcdsaAuthority", b"Authorities", &[]));

		<pallet_balances::Locks<Runtime>>::iter_values().for_each(|v| {
			assert!(!v.is_empty());
		});

		Ok(())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		migrate()
	}
}

fn migrate() -> frame_support::weights::Weight {
	let mut r = 0;
	let mut w = 101;
	let _ =
		migration::clear_storage_prefix(b"MessageGadget", b"CommitmentContract", &[], None, None);
	let lock_ids = [
		// Democracy lock.
		*b"democrac",
		// Fee market lock.
		*b"da/feecr",
	];

	<pallet_balances::Locks<Runtime>>::iter().for_each(|(k, mut v)| {
		if v.is_empty() {
			// Clear the storage entry if the vector is empty.

			<pallet_balances::Locks<Runtime>>::remove(k);

			w += 1;
		} else {
			// Find matching lock ids and remove them.

			let mut changed = false;

			v.retain(|l| {
				if lock_ids.contains(&l.id) {
					// Mark as changed, the storage entry needs to be updated.
					changed = true;

					// To remove.
					false
				} else {
					// To keep.
					true
				}
			});

			if changed {
				<pallet_balances::Locks<Runtime>>::insert(k, v);

				w += 1;
			}
		}

		r += 1;
	});

	w += migration_helper::PalletCleaner {
		name: b"EcdsaAuthority",
		values: &[
			b"Authorities",
			b"NextAuthorities",
			b"Nonce",
			b"AuthoritiesChangeToSign",
			b"MessageRootToSign",
		],
		maps: &[],
	}
	.remove_all();

	let _ = migration::clear_storage_prefix(
		b"BridgeKusamaGrandpa",
		b"ImportedHeaders",
		&[],
		Some(100),
		None,
	);

	// frame_support::weights::Weight::zero()
	<Runtime as frame_system::Config>::DbWeight::get().reads_writes(r as _, w as _)
}
