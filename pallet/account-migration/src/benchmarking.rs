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
use crate::*;
// polkadot-sdk
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_std::prelude::*;

#[v2::benchmarks]
mod benchmarks {
	// darwinia
	use super::*;

	fn preset_data<T>(from: &AccountId32)
	where
		T: Config,
	{
		const AMOUNT: Balance = 1_024;

		let encoded_kton_id = KTON_ID.encode();

		<pallet_balances::TotalIssuance<T>>::put(AMOUNT);

		if let Some(mut asset_details) = migration::take_storage_value::<AssetDetails>(
			b"Assets",
			b"Asset",
			&Blake2_128Concat::hash(&encoded_kton_id),
		) {
			asset_details.supply = AMOUNT;

			migration::put_storage_value(
				b"Assets",
				b"Asset",
				&Blake2_128Concat::hash(&encoded_kton_id),
				asset_details,
			);
		}

		<Accounts<T>>::insert(
			from,
			AccountInfo {
				data: AccountData { free: AMOUNT, ..Default::default() },
				..Default::default()
			},
		);
		<KtonAccounts<T>>::insert(
			from,
			AssetAccount {
				balance: AMOUNT,
				is_frozen: Default::default(),
				reason: ExistenceReason::Sufficient,
				extra: Default::default(),
			},
		);
		<Deposits<T>>::insert(
			from,
			vec![
				Deposit {
					id: Default::default(),
					value: 1,
					start_time: Default::default(),
					expired_time: Default::default(),
					in_use: Default::default(),
				};
				512
			],
		);
	}

	#[benchmark]
	fn migrate() {
		let from = [0; 32].into();
		let to = [0; 20].into();

		// Worst-case scenario:
		//
		// Migrate all kinds of data.
		preset_data::<T>(&from);

		#[extrinsic_call]
		_(RawOrigin::None, from, to, [0; 64]);
	}

	frame_benchmarking::impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext(),
		crate::mock::Runtime
	);
}
