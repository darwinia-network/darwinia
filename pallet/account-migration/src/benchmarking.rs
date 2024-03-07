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
// substrate
use frame_benchmarking::v2;
use frame_system::RawOrigin;
use pallet_identity::{IdentityInfo, Judgement};
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
		<Identities<T>>::insert(
			from,
			Registration {
				judgements: BoundedVec::truncate_from(vec![
					(Default::default(), Judgement::Unknown);
					<T as pallet_identity::Config>::MaxRegistrars::get()
						as usize
				]),
				deposit: Default::default(),
				info: IdentityInfo {
					additional: BoundedVec::truncate_from(vec![
						(
							Default::default(),
							Default::default()
						);
						<T as pallet_identity::Config>::MaxAdditionalFields::get()
							as usize
					]),
					display: Default::default(),
					legal: Default::default(),
					web: Default::default(),
					riot: Default::default(),
					email: Default::default(),
					pgp_fingerprint: Some(Default::default()),
					image: Default::default(),
					twitter: Default::default(),
				},
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
				<T as dp_deposit::Config>::MaxDeposits::get() as usize
			],
		);
		<Ledgers<T>>::insert(
			from,
			Ledger {
				staked_ring: 1,
				staked_kton: 1,
				staked_deposits: BoundedVec::truncate_from(vec![
					Default::default();
					<T as dp_deposit::Config>::MaxDeposits::get()
						as usize
				]),
				unstaking_ring: BoundedVec::truncate_from(vec![
					(
						Default::default(),
						Default::default()
					);
					<T as dp_staking::Config>::MaxUnstakings::get()
						as usize
				]),
				unstaking_kton: BoundedVec::truncate_from(vec![
					(
						Default::default(),
						Default::default()
					);
					<T as dp_staking::Config>::MaxUnstakings::get()
						as usize
				]),
				unstaking_deposits: Default::default(),
			},
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

	fn other_multisig_members<const N: usize, A>(count: u32) -> Vec<A>
	where
		A: From<[u8; N]>,
	{
		(0..count).map(|i| [i as u8; N].into()).collect()
	}

	#[benchmark]
	fn migrate_multisig(x: Linear<0, 99>, y: Linear<0, 99>, z: Linear<0, 99>) {
		let from = AccountId32::from([0; 32]);
		// Worst-case scenario:
		//
		// 100 size multisig.
		let other_members = other_multisig_members(x);
		let (_, multisig) = multisig_of(from.clone(), other_members.clone(), y as _);
		let to = [0; 20].into();
		let new_multisig_params = if z == 0 {
			None
		} else {
			Some(MultisigParams {
				address: [0; 20].into(),
				members: other_multisig_members::<20, _>(z),
				threshold: Default::default(),
			})
		};

		preset_data::<T>(&multisig);

		#[extrinsic_call]
		_(RawOrigin::None, from, other_members, y as _, to, [0; 64], new_multisig_params);
	}

	#[benchmark]
	fn complete_multisig_migration() {
		let from = AccountId32::from([0; 32]);
		// Worst-case scenario:
		//
		// 100 size multisig.
		let other_members = other_multisig_members(99);
		let (_, multisig) = multisig_of(from.clone(), other_members.clone(), 100);
		let to = [0; 20].into();

		<Pallet<T>>::migrate_multisig(
			RawOrigin::None.into(),
			from.clone(),
			other_members,
			100,
			to,
			[0; 64],
			None,
		)
		.unwrap();

		#[extrinsic_call]
		_(RawOrigin::None, multisig, from, [0; 64]);
	}

	frame_benchmarking::impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext(),
		crate::mock::Runtime
	);
}
