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
	if let Ok(a) =
		array_bytes::hex_n_into::<_, AccountId, 20>("0xacfa39b864e42d1bd3792783a571d2958af0bf1f")
	{
		let mut l = <pallet_balances::Locks<Runtime>>::get(a);

		if let Some(i) = l.iter().position(|l| l.id == &*"phrelect"[..]) {
			l.remove(i);

			if l.is_empty() {
				<pallet_balances::Locks<Runtime>>::remove(a);
			} else {
				<pallet_balances::Locks<Runtime>>::insert(a, l);
			}
		}
	}

	[
		"0xd891ce6a97b4f01a8b9b36d0298aa3631fe2eef5",
		"0x88a39b052d477cfde47600a7c9950a441ce61cb4",
		"0x0a1287977578f888bdc1c7627781af1cc000e6ab",
		"0x0b001c95e86d64c1ad6e43944c568a6c31b53887",
		"0x7ae2a0914db8bfbdad538b0eac3fa473a0e07843",
		"0xacfa39b864e42d1bd3792783a571d2958af0bf1f",
		"0x5af9a1be7bc22f9a6b2ce90acd69c23dceeb23c2",
		"0x1678a973ae9750d25c126cdbce891bb8cfacd520",
		"0x4ed7ae57608cf4f60753cde4f49cf821c293ed2a",
		"0x5b7544b3f6abd9e03fba494796b1ee6f9543e2e4",
		"0x44cda595218ddb3810fb66c2e982f50ea00255ee",
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

	// frame_support::weights::Weight::zero()
	<Runtime as frame_system::Config>::DbWeight::get().reads_writes(1, 23)
}
