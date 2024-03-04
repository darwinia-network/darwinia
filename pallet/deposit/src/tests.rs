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
use crate::{
	mock::{Deposit, *},
	Deposit as DepositS, *,
};
use darwinia_staking_traits::Stake;
// substrate
use frame_support::{assert_noop, assert_ok};
use sp_runtime::TokenError;

#[test]
fn lock_should_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(System::account(1).consumers, 0);
		assert_eq!(Balances::free_balance(darwinia_deposit::account_id::<u32>()), 0);
		assert_eq!(Balances::free_balance(1), 1_000 * UNIT);
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), 10 * UNIT, 1));
		assert_eq!(System::account(1).consumers, 1);
		assert_eq!(Balances::free_balance(darwinia_deposit::account_id::<u32>()), 10 * UNIT);
		assert_eq!(Balances::free_balance(1), 990 * UNIT);
	});
}

#[test]
fn deposit_interest_should_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(Assets::balance(0, 1), 0);
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_eq!(Assets::balance(0, 1), 7_614_213_197_969);

		assert_eq!(Assets::balance(0, 2), 0);
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(2), 1000 * UNIT, MAX_LOCKING_MONTHS));
		assert_eq!(Assets::balance(0, 2), 364_467_005_076_142_131);
	});
}

#[test]
fn unique_identity_should_work() {
	new_test_ext().execute_with(|| {
		assert!(Deposit::deposit_of(1).is_none());
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), 2 * UNIT, 2));
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), 3 * UNIT, 1));
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), 4 * UNIT, 2));
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), 5 * UNIT, 1));
		assert_eq!(
			Deposit::deposit_of(1).unwrap().as_slice(),
			&[
				DepositS {
					id: 0,
					value: UNIT,
					start_time: 0,
					expired_time: MILLISECS_PER_MONTH,
					in_use: false
				},
				DepositS {
					id: 1,
					value: 2 * UNIT,
					start_time: 0,
					expired_time: 2 * MILLISECS_PER_MONTH,
					in_use: false
				},
				DepositS {
					id: 2,
					value: 3 * UNIT,
					start_time: 0,
					expired_time: MILLISECS_PER_MONTH,
					in_use: false
				},
				DepositS {
					id: 3,
					value: 4 * UNIT,
					start_time: 0,
					expired_time: 2 * MILLISECS_PER_MONTH,
					in_use: false
				},
				DepositS {
					id: 4,
					value: 5 * UNIT,
					start_time: 0,
					expired_time: MILLISECS_PER_MONTH,
					in_use: false
				}
			]
		);

		efflux(MILLISECS_PER_MONTH);
		assert_ok!(Deposit::claim(RuntimeOrigin::signed(1)));

		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), 6 * UNIT, 1));
		assert_eq!(
			Deposit::deposit_of(1).unwrap().as_slice(),
			&[
				DepositS {
					id: 0,
					value: 6 * UNIT,
					start_time: MILLISECS_PER_MONTH,
					expired_time: 2 * MILLISECS_PER_MONTH,
					in_use: false
				},
				DepositS {
					id: 1,
					value: 2 * UNIT,
					start_time: 0,
					expired_time: 2 * MILLISECS_PER_MONTH,
					in_use: false
				},
				DepositS {
					id: 3,
					value: 4 * UNIT,
					start_time: 0,
					expired_time: 2 * MILLISECS_PER_MONTH,
					in_use: false
				},
			]
		);

		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), 7 * UNIT, 1));
		assert_eq!(
			Deposit::deposit_of(1).unwrap().as_slice(),
			&[
				DepositS {
					id: 0,
					value: 6 * UNIT,
					start_time: MILLISECS_PER_MONTH,
					expired_time: 2 * MILLISECS_PER_MONTH,
					in_use: false
				},
				DepositS {
					id: 1,
					value: 2 * UNIT,
					start_time: 0,
					expired_time: 2 * MILLISECS_PER_MONTH,
					in_use: false
				},
				DepositS {
					id: 2,
					value: 7 * UNIT,
					start_time: MILLISECS_PER_MONTH,
					expired_time: 2 * MILLISECS_PER_MONTH,
					in_use: false
				},
				DepositS {
					id: 3,
					value: 4 * UNIT,
					start_time: 0,
					expired_time: 2 * MILLISECS_PER_MONTH,
					in_use: false
				},
			]
		);

		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), 8 * UNIT, 1));
		assert_eq!(
			Deposit::deposit_of(1).unwrap().as_slice(),
			&[
				DepositS {
					id: 0,
					value: 6 * UNIT,
					start_time: MILLISECS_PER_MONTH,
					expired_time: 2 * MILLISECS_PER_MONTH,
					in_use: false
				},
				DepositS {
					id: 1,
					value: 2 * UNIT,
					start_time: 0,
					expired_time: 2 * MILLISECS_PER_MONTH,
					in_use: false
				},
				DepositS {
					id: 2,
					value: 7 * UNIT,
					start_time: MILLISECS_PER_MONTH,
					expired_time: 2 * MILLISECS_PER_MONTH,
					in_use: false
				},
				DepositS {
					id: 3,
					value: 4 * UNIT,
					start_time: 0,
					expired_time: 2 * MILLISECS_PER_MONTH,
					in_use: false
				},
				DepositS {
					id: 4,
					value: 8 * UNIT,
					start_time: MILLISECS_PER_MONTH,
					expired_time: 2 * MILLISECS_PER_MONTH,
					in_use: false
				},
			]
		);
	});
}

#[test]
fn expire_time_should_work() {
	new_test_ext().execute_with(|| {
		(1..=8).for_each(|_| {
			assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
			efflux(MILLISECS_PER_MONTH);
		});
		assert_eq!(
			Deposit::deposit_of(1).unwrap().as_slice(),
			(1..=8)
				.map(|i| DepositS {
					id: i - 1,
					value: UNIT,
					start_time: (i - 1) as Moment * MILLISECS_PER_MONTH,
					expired_time: i as Moment * MILLISECS_PER_MONTH,
					in_use: false
				})
				.collect::<Vec<_>>()
				.as_slice()
		);
	});
}

#[test]
fn lock_should_fail() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Deposit::lock(RuntimeOrigin::signed(1), 0, 0),
			<Error<Runtime>>::LockAtLeastSome
		);

		assert_noop!(
			Deposit::lock(RuntimeOrigin::signed(1), UNIT, 0),
			<Error<Runtime>>::LockAtLeastOneMonth
		);

		assert_noop!(
			Deposit::lock(RuntimeOrigin::signed(1), UNIT, 37),
			<Error<Runtime>>::LockAtMostThirtySixMonths
		);

		(0..<<Runtime as Config>::MaxDeposits as Get<_>>::get()).for_each(|_| {
			assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		});
		assert_noop!(
			Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1),
			<Error<Runtime>>::ExceedMaxDeposits
		);

		assert_noop!(
			Deposit::lock(RuntimeOrigin::signed(2), 2_001 * UNIT, 1),
			TokenError::FundsUnavailable
		);
	});
}

#[test]
fn claim_should_work() {
	new_test_ext().execute_with(|| {
		assert!(Deposit::deposit_of(1).is_none());
		assert_ok!(Deposit::claim(RuntimeOrigin::signed(1)));
		assert!(Deposit::deposit_of(1).is_none());

		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert!(Deposit::deposit_of(1).is_some());

		efflux(MILLISECS_PER_MONTH - 1);
		assert_eq!(System::account(1).consumers, 1);
		assert_ok!(Deposit::claim(RuntimeOrigin::signed(1)));
		assert_eq!(System::account(1).consumers, 1);
		assert!(Deposit::deposit_of(1).is_some());

		efflux(MILLISECS_PER_MONTH);
		assert_eq!(System::account(1).consumers, 1);
		assert_ok!(Deposit::claim(RuntimeOrigin::signed(1)));
		assert_eq!(System::account(1).consumers, 0);
		assert!(Deposit::deposit_of(1).is_none());

		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_ok!(Deposit::stake(&1, 0));
		efflux(2 * MILLISECS_PER_MONTH);
		assert_eq!(System::account(1).consumers, 1);
		assert_ok!(Deposit::claim(RuntimeOrigin::signed(1)));
		assert_eq!(System::account(1).consumers, 1);
		assert!(Deposit::deposit_of(1).is_some());

		assert_ok!(Deposit::unstake(&1, 0));
		assert_eq!(System::account(1).consumers, 1);
		assert_ok!(Deposit::claim(RuntimeOrigin::signed(1)));
		assert_eq!(System::account(1).consumers, 0);
		assert!(Deposit::deposit_of(1).is_none());
	});
}

#[test]
fn claim_with_penalty_should_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(&1), 1_000 * UNIT);
		assert!(Deposit::deposit_of(1).is_none());
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_eq!(Balances::free_balance(&1), 999 * UNIT);
		assert!(Deposit::deposit_of(1).is_some());

		assert_noop!(
			Deposit::claim_with_penalty(RuntimeOrigin::signed(1), 0),
			<pallet_assets::Error<Runtime>>::BalanceLow
		);

		assert_ok!(KtonMinting::mint(&1, UNIT));
		assert_ok!(Deposit::claim_with_penalty(RuntimeOrigin::signed(1), 0));
		assert_eq!(Balances::free_balance(&1), 1_000 * UNIT);
		assert_eq!(Assets::balance(0, 1), 999_984_771_573_604_062);
		assert!(Deposit::deposit_of(1).is_none());

		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		efflux(MILLISECS_PER_MONTH);
		assert!(Deposit::deposit_of(1).is_some());

		assert_noop!(
			Deposit::claim_with_penalty(RuntimeOrigin::signed(1), 0),
			<Error<Runtime>>::DepositAlreadyExpired
		);
	});
}
