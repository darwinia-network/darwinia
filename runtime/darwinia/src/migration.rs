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

		assert!(migration::storage_iter::<()>(b"Vesting", b"Vesting").count() == 1);
		assert_eq!(
			Balances::locks(
				// 0x081cbab52e2dbcd52f441c7ae9ad2a3be42e2284.
				AccountId::from([
					8, 28, 186, 181, 46, 45, 188, 213, 47, 68, 28, 122, 233, 173, 42, 59, 228, 46,
					34, 132,
				]),
			)
			.into_iter()
			.map(|l| l.id)
			.collect::<Vec<_>>(),
			[*b"vesting "],
		);

		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		log::info!("post");

		darwinia_staking::migration::post_check::<Runtime>();

		assert!(migration::storage_iter::<()>(b"Vesting", b"Vesting").count() == 0);
		assert!(Balances::locks(
			// 0x081cbab52e2dbcd52f441c7ae9ad2a3be42e2284.
			AccountId::from([
				8, 28, 186, 181, 46, 45, 188, 213, 47, 68, 28, 122, 233, 173, 42, 59, 228, 46, 34,
				132,
			]),
		)
		.is_empty());

		Ok(())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		migrate()
	}
}

fn migrate() -> frame_support::weights::Weight {
	// polkadot-sdk
	use frame_support::{traits::LockableCurrency, PalletId};
	use sp_runtime::traits::AccountIdConversion;

	let (r, w) = darwinia_staking::migration::migrate::<Runtime>();
	let _ = migration::clear_storage_prefix(b"Vesting", b"Vesting", &[], None, None);

	Balances::remove_lock(
		*b"vesting ",
		// 0x081cbab52e2dbcd52f441c7ae9ad2a3be42e2284.
		&AccountId::from([
			8, 28, 186, 181, 46, 45, 188, 213, 47, 68, 28, 122, 233, 173, 42, 59, 228, 46, 34, 132,
		]),
	);

	let _ = Balances::transfer_all(
		RuntimeOrigin::signed(PalletId(*b"dar/depo").into_account_truncating()),
		Treasury::account_id(),
		false,
	);

	<Runtime as frame_system::Config>::DbWeight::get().reads_writes(r, w + 10)
}
