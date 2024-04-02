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
use dc_types::UNIT;
// substrate
use frame_benchmarking::v2;
use frame_system::RawOrigin;
use sp_runtime::SaturatedConversion;
use sp_std::prelude::*;

#[v2::benchmarks]
mod benchmarks {
	// darwinia
	use super::*;

	#[benchmark]
	fn lock() {
		let a = frame_benchmarking::whitelisted_caller();
		let max_deposits = T::MaxDeposits::get();

		// Remove `+ 1` after https://github.com/paritytech/substrate/pull/13655.
		T::Ring::make_free_balance_be(&a, max_deposits as Balance * UNIT + 1);

		// Worst-case scenario:
		//
		// Calculate the last deposit's id.
		(0..max_deposits - 1).for_each(|_| {
			<Pallet<T>>::lock(RawOrigin::Signed(a.clone()).into(), UNIT, MAX_LOCKING_MONTHS)
				.unwrap()
		});

		#[extrinsic_call]
		_(RawOrigin::Signed(a), UNIT, MAX_LOCKING_MONTHS);
	}

	#[benchmark]
	fn claim() {
		let a = frame_benchmarking::whitelisted_caller();
		let max_deposits = T::MaxDeposits::get();

		// Remove `+ 1` after https://github.com/paritytech/substrate/pull/13655.
		T::Ring::make_free_balance_be(&a, max_deposits as Balance * UNIT + 1);

		(0..max_deposits).for_each(|_| {
			<Pallet<T>>::lock(RawOrigin::Signed(a.clone()).into(), UNIT, MAX_LOCKING_MONTHS)
				.unwrap()
		});

		// Worst-case scenario:
		//
		// Let all locks be expired.
		<pallet_timestamp::Pallet<T>>::set_timestamp(
			<pallet_timestamp::Pallet<T>>::now()
				+ (MAX_LOCKING_MONTHS as Moment * MILLISECS_PER_MONTH).saturated_into(),
		);

		assert_eq!(<Pallet<T>>::deposit_of(&a).unwrap().len(), max_deposits as usize);

		#[extrinsic_call]
		_(RawOrigin::Signed(a.clone()));

		assert!(<Pallet<T>>::deposit_of(&a).is_none());
	}

	#[benchmark]
	fn claim_with_penalty() {
		let a = frame_benchmarking::whitelisted_caller();
		let max_deposits = T::MaxDeposits::get();

		// Remove `+ 1` after https://github.com/paritytech/substrate/pull/13655.
		T::Ring::make_free_balance_be(&a, max_deposits as Balance * UNIT + 1);
		T::Kton::mint(&a, UNIT).unwrap();

		(0..max_deposits).for_each(|_| {
			<Pallet<T>>::lock(RawOrigin::Signed(a.clone()).into(), UNIT, MAX_LOCKING_MONTHS)
				.unwrap()
		});

		// Worst-case scenario:
		//
		// Remove the head item from a 'full-size' bounded vector.
		{
			let ds = <Pallet<T>>::deposit_of(&a).unwrap();

			assert_eq!(ds.len(), max_deposits as usize);
			assert_eq!(ds[0].id, 0);
		}

		#[extrinsic_call]
		_(RawOrigin::Signed(a.clone()), 0);

		{
			let ds = <Pallet<T>>::deposit_of(&a).unwrap();

			assert_eq!(ds.len(), max_deposits as usize - 1);
			assert_eq!(ds[0].id, 1);
		}
	}

	frame_benchmarking::impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext(),
		crate::mock::Runtime
	);
}
