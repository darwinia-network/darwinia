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
use crate::{Deposit, *};
// polkadot-sdk
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[v2::benchmarks]
mod benchmarks {
	// darwinia
	use super::*;

	#[benchmark]
	fn migrate_for() {
		let a = frame_benchmarking::whitelisted_caller::<T::AccountId>();
		let a_ = a.clone();

		<Pallet<T>>::set_deposit_contract(RawOrigin::Root.into(), a.clone()).unwrap();

		T::Ring::make_free_balance_be(&T::Treasury::get(), 2 << 126);

		// Worst-case scenario:
		//
		// Max deposit items to be migrated.
		<Deposits<T>>::insert(&a, {
			let mut v = BoundedVec::new();

			(0..512).for_each(|id| {
				v.try_push(Deposit {
					id,
					value: 1,
					start_time: 0,
					expired_time: Moment::MAX,
					in_use: false,
				})
				.unwrap();
			});

			v
		});

		#[extrinsic_call]
		_(RawOrigin::Signed(a), a_);
	}

	#[benchmark]
	fn set_deposit_contract() {
		let a = frame_benchmarking::whitelisted_caller();

		// Worst-case scenario:
		//
		// Set successfully.
		#[extrinsic_call]
		_(RawOrigin::Root, a);
	}

	frame_benchmarking::impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext(),
		crate::mock::Runtime
	);
}
