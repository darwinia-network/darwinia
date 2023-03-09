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
use frame_support::log;

#[cfg(feature = "try-runtime")]
const ERROR_ACCOUNT: &str = "0xa847fbb7ce32a41fbea2216c7073752bb13dd6bfae44bc0f726e020452c2105b";

pub struct CustomOnRuntimeUpgrade;
impl frame_support::traits::OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		log::info!("Pre-check account: {ERROR_ACCOUNT}");

		let a = array_bytes::hex_n_into_unchecked::<_, sp_runtime::AccountId32, 32>(ERROR_ACCOUNT);

		assert_eq!(
			AccountMigration::ledger_of(&a).unwrap(),
			darwinia_staking::Ledger::<Runtime> {
				staked_ring: 3_190_000_000_000_000_000_000,
				staked_kton: 9_000_000_000_000_000,
				staked_deposits: frame_support::BoundedVec::truncate_from(vec![0, 1, 2]),
				unstaking_ring: frame_support::BoundedVec::truncate_from(vec![(
					10_000_000_000_000_000_000,
					0
				)]),
				unstaking_kton: frame_support::BoundedVec::truncate_from(vec![(
					1_000_000_000_000_000,
					0
				)]),
				unstaking_deposits: frame_support::BoundedVec::default()
			}
		);

		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
		log::info!("Post-check account: {ERROR_ACCOUNT}");

		let a = array_bytes::hex_n_into_unchecked::<_, sp_runtime::AccountId32, 32>(ERROR_ACCOUNT);

		assert_eq!(
			AccountMigration::ledger_of(&a).unwrap(),
			darwinia_staking::Ledger::<Runtime> {
				staked_ring: 3_190_000_000_000_000_000_000
					- AccountMigration::deposit_of(&a)
						.unwrap()
						.into_iter()
						.map(|d| d.value)
						.sum::<Balance>(),
				staked_kton: 9_000_000_000_000_000,
				staked_deposits: frame_support::BoundedVec::truncate_from(vec![0, 1, 2]),
				unstaking_ring: frame_support::BoundedVec::default(),
				unstaking_kton: frame_support::BoundedVec::default(),
				unstaking_deposits: frame_support::BoundedVec::default()
			}
		);

		Ok(())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		migrate()
	}
}

fn migrate() -> frame_support::weights::Weight {
	<darwinia_account_migration::Ledgers<Runtime>>::translate(
		|k, mut v: darwinia_staking::Ledger<Runtime>| {
			if let Some(ds) = <darwinia_account_migration::Deposits<Runtime>>::get(k) {
				v.staked_ring -= ds.into_iter().map(|d| d.value).sum::<Balance>();
			}

			v.unstaking_ring.retain(|u| u.1 != 0);
			v.unstaking_kton.retain(|u| u.1 != 0);

			Some(v)
		},
	);

	// frame_support::weights::Weight::zero()
	RuntimeBlockWeights::get().max_block
}
