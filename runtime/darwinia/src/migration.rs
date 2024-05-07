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
	[
		"0x71571c42067900bfb7ca8b51fccc07ef77074aea",
		"0x0a1287977578f888bdc1c7627781af1cc000e6ab",
		"0x0b001c95e86d64c1ad6e43944c568a6c31b53887",
		"0xfa5727be643dba6599fc7f812fe60da3264a8205",
		"0x5af9a1be7bc22f9a6b2ce90acd69c23dceeb23c2",
		"0x1678a973ae9750d25c126cdbce891bb8cfacd520",
		"0x5dd68958e07cec3f65489db8983ad737c37e0646",
		"0xf11d8d9412fc6b90242e17af259cf7bd1eaa416b",
		"0xdca962b899641d60ccf7268a2260f20b6c01c06d",
	]
	.iter()
	.filter_map(|a| array_bytes::hex_n_into::<_, AccountId, 20>(a).ok())
	.for_each(|a| {
		let freeze = <pallet_balances::Freezes<Runtime>>::get(a)
			.into_iter()
			.map(|f| f.amount)
			.max()
			.unwrap_or(0);
		let frozen = <pallet_balances::Locks<Runtime>>::get(a)
			.into_iter()
			.map(|l| l.amount)
			.max()
			.unwrap_or(0);
		let frozen = freeze.max(frozen);
		let _ = <frame_system::Account<Runtime>>::try_mutate(a, |a| {
			if a.data.frozen == frozen {
				Err(())
			} else {
				a.data.frozen = frozen;

				Ok(())
			}
		});
	});

	let _ = migration::clear_storage_prefix(
		b"BridgeKusamaGrandpa",
		b"ImportedHeaders",
		&[],
		Some(100),
		None,
	);

	// frame_support::weights::Weight::zero()
	<Runtime as frame_system::Config>::DbWeight::get().reads_writes(0, 118)
}
