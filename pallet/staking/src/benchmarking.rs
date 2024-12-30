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

	#[benchmark]
	fn unstake_all_for() {
		let a = frame_benchmarking::whitelisted_caller::<T::AccountId>();
		let a_cloned = a.clone();

		T::Currency::make_free_balance_be(&account_id(), 1);
		<Ledgers<T>>::insert(&a, Ledger { ring: 1, deposits: BoundedVec::new() });

		#[extrinsic_call]
		_(RawOrigin::Signed(a), a_cloned);
	}

	#[benchmark]
	fn allocate_ring_staking_reward_of() {
		let a = frame_benchmarking::whitelisted_caller::<T::AccountId>();
		let a_cloned = a.clone();

		<PendingRewards<T>>::insert(&a, 1);

		#[extrinsic_call]
		_(RawOrigin::Signed(a), a_cloned);
	}

	#[benchmark]
	fn set_ring_staking_contract() {
		// Worst-case scenario:
		//
		// Set successfully.
		#[extrinsic_call]
		_(RawOrigin::Root, frame_benchmarking::whitelisted_caller::<T::AccountId>());
	}

	#[benchmark]
	fn set_kton_staking_contract() {
		// Worst-case scenario:
		//
		// Set successfully.
		#[extrinsic_call]
		_(RawOrigin::Root, frame_benchmarking::whitelisted_caller::<T::AccountId>());
	}

	#[benchmark]
	fn set_collator_count() {
		// Worst-case scenario:
		//
		// Set successfully.
		#[extrinsic_call]
		_(RawOrigin::Root, 1);
	}

	frame_benchmarking::impl_benchmark_test_suite!(
		Pallet,
		crate::mock::ExtBuilder.build(),
		crate::mock::Runtime
	);
}
