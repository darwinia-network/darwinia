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
use darwinia_deposit::SimpleAsset;
use dc_primitives::UNIT;
// polkadot-sdk
use frame_benchmarking::v2;
use frame_system::RawOrigin;
use sp_std::prelude::*;

#[v2::benchmarks]
mod benchmarks {
	// darwinia
	use super::*;
	// polkadot-sdk
	use frame_support::traits::Currency;

	fn deposit_for<T>(who: &T::AccountId, count: u32) -> Vec<DepositId<T>>
	where
		T: Config + darwinia_deposit::Config,
	{
		(0..count.min(<<T as darwinia_deposit::Config>::MaxDeposits>::get()) as u16)
			.map(|x| {
				<darwinia_deposit::Pallet<T>>::lock(
					RawOrigin::Signed(who.to_owned()).into(),
					UNIT,
					1,
				)
				.unwrap();

				x.into()
			})
			.collect()
	}

	#[benchmark]
	fn stake(x: Linear<0, 1_023>) {
		let a = frame_benchmarking::whitelisted_caller();

		// Remove `+ 1` after https://github.com/paritytech/substrate/pull/13655.
		<T as darwinia_deposit::Config>::Ring::make_free_balance_be(&a, 1_024 * UNIT + 1);
		<T as darwinia_deposit::Config>::Kton::mint(&a, UNIT).unwrap();

		let deposits = deposit_for::<T>(&a, x);

		// Worst-case scenario:
		//
		// The total number of deposit items has reached `darwinia_deposits::Config::MaxDeposits`.
		#[extrinsic_call]
		_(RawOrigin::Signed(a), UNIT, deposits);
	}

	#[benchmark]
	fn unstake(x: Linear<0, 1_023>) {
		let a = frame_benchmarking::whitelisted_caller();

		// Remove `+ 1` after https://github.com/paritytech/substrate/pull/13655.
		<T as darwinia_deposit::Config>::Ring::make_free_balance_be(&a, 1_024 * UNIT + 1);
		<T as darwinia_deposit::Config>::Kton::mint(&a, UNIT).unwrap();

		let deposits = deposit_for::<T>(&a, x);

		<Pallet<T>>::stake(RawOrigin::Signed(a.clone()).into(), UNIT, deposits.clone()).unwrap();

		// Worst-case scenario:
		//
		// The total number of deposit items has reached `darwinia_deposits::Config::MaxDeposits`.
		#[extrinsic_call]
		_(RawOrigin::Signed(a), UNIT, deposits);
	}

	#[benchmark]
	fn collect() {
		let a = frame_benchmarking::whitelisted_caller();

		// Worst-case scenario:
		//
		// None.
		#[extrinsic_call]
		_(RawOrigin::Signed(a), Default::default());
	}

	#[benchmark]
	fn nominate() {
		let a = frame_benchmarking::whitelisted_caller::<T::AccountId>();
		let a_cloned = a.clone();

		// Remove `+ 1` after https://github.com/paritytech/substrate/pull/13655.
		<T as darwinia_deposit::Config>::Ring::make_free_balance_be(&a, UNIT + 1);

		<Pallet<T>>::stake(RawOrigin::Signed(a.clone()).into(), UNIT, Default::default()).unwrap();
		<Pallet<T>>::collect(RawOrigin::Signed(a.clone()).into(), Default::default()).unwrap();

		// Worst-case scenario:
		//
		// Nominate the target collator successfully.
		#[extrinsic_call]
		_(RawOrigin::Signed(a), a_cloned);
	}

	#[benchmark]
	fn chill() {
		let a = frame_benchmarking::whitelisted_caller::<T::AccountId>();

		// Remove `+ 1` after https://github.com/paritytech/substrate/pull/13655.
		<T as darwinia_deposit::Config>::Ring::make_free_balance_be(&a, UNIT + 1);

		<Pallet<T>>::stake(RawOrigin::Signed(a.clone()).into(), UNIT, Default::default()).unwrap();
		<Pallet<T>>::collect(RawOrigin::Signed(a.clone()).into(), Default::default()).unwrap();
		<Pallet<T>>::nominate(RawOrigin::Signed(a.clone()).into(), a.clone()).unwrap();

		// Worst-case scenario:
		//
		// Collect and nominate at the same time.
		#[extrinsic_call]
		_(RawOrigin::Signed(a));
	}

	#[benchmark]
	fn payout() {
		let a = frame_benchmarking::whitelisted_caller::<T::AccountId>();
		let sender = a.clone();

		call_on_exposure!(<Previous<T>>::insert(
			&a,
			Exposure {
				commission: Perbill::zero(),
				vote: 32,
				nominators: (0..32)
					.map(|i| IndividualExposure {
						who: frame_benchmarking::account("", i, i),
						vote: 1,
					})
					.collect(),
			},
		))
		.unwrap();
		<PendingRewards<T>>::insert(&a, 500);

		#[extrinsic_call]
		_(RawOrigin::Signed(sender), a);
	}

	#[benchmark]
	fn set_rate_limit() {
		// Worst-case scenario:
		//
		// Set successfully.
		#[extrinsic_call]
		_(RawOrigin::Root, 1);
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
		crate::mock::ExtBuilder::default().build(),
		crate::mock::Runtime
	);
}
