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

// core
use core::time::Duration;
// crates.io
use ethereum::TransactionSignature;
// darwinia
use crate::{mock::*, MigrationCurve, *};
use darwinia_deposit::Error as DepositError;
use dc_types::UNIT;
// substrate
use frame_support::{assert_noop, assert_ok, BoundedVec};
use sp_runtime::{assert_eq_error_rate, DispatchError, Perbill};
use substrate_test_utils::assert_eq_uvec;

#[test]
fn exposure_cache_states_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		#[allow(deprecated)]
		{
			<ExposureCacheStates<Runtime>>::kill();
			<ExposureCache0<Runtime>>::remove_all(None);
			<ExposureCache1<Runtime>>::remove_all(None);
			<ExposureCache2<Runtime>>::remove_all(None);

			let e = Exposure {
				commission: Default::default(),
				vote: Default::default(),
				nominators: Default::default(),
			};

			<ExposureCache0<Runtime>>::insert(0, e.clone());
			<ExposureCache1<Runtime>>::insert(1, e.clone());
			<ExposureCache2<Runtime>>::insert(2, e);
		}

		assert!(call_on_exposure!(<Previous<Runtime>>::get(0).is_some()).unwrap());
		assert!(call_on_exposure!(<Previous<Runtime>>::get(1).is_none()).unwrap());
		assert!(call_on_exposure!(<Previous<Runtime>>::get(2).is_none()).unwrap());
		assert!(call_on_exposure!(<Current<Runtime>>::get(0).is_none()).unwrap());
		assert!(call_on_exposure!(<Current<Runtime>>::get(1).is_some()).unwrap());
		assert!(call_on_exposure!(<Current<Runtime>>::get(2).is_none()).unwrap());
		assert!(call_on_exposure!(<Next<Runtime>>::get(0).is_none()).unwrap());
		assert!(call_on_exposure!(<Next<Runtime>>::get(1).is_none()).unwrap());
		assert!(call_on_exposure!(<Next<Runtime>>::get(2).is_some()).unwrap());
		assert_eq!(
			<ExposureCacheStates<Runtime>>::get(),
			(ExposureCacheState::Previous, ExposureCacheState::Current, ExposureCacheState::Next)
		);

		Staking::shift_exposure_cache_states();

		assert!(call_on_exposure!(<Previous<Runtime>>::get(0).is_none()).unwrap());
		assert!(call_on_exposure!(<Previous<Runtime>>::get(1).is_some()).unwrap());
		assert!(call_on_exposure!(<Previous<Runtime>>::get(2).is_none()).unwrap());
		assert!(call_on_exposure!(<Current<Runtime>>::get(0).is_none()).unwrap());
		assert!(call_on_exposure!(<Current<Runtime>>::get(1).is_none()).unwrap());
		assert!(call_on_exposure!(<Current<Runtime>>::get(2).is_some()).unwrap());
		assert!(call_on_exposure!(<Next<Runtime>>::get(0).is_some()).unwrap());
		assert!(call_on_exposure!(<Next<Runtime>>::get(1).is_none()).unwrap());
		assert!(call_on_exposure!(<Next<Runtime>>::get(2).is_none()).unwrap());
		assert_eq!(
			<ExposureCacheStates<Runtime>>::get(),
			(ExposureCacheState::Next, ExposureCacheState::Previous, ExposureCacheState::Current)
		);

		Staking::shift_exposure_cache_states();

		assert!(call_on_exposure!(<Previous<Runtime>>::get(0).is_none()).unwrap());
		assert!(call_on_exposure!(<Previous<Runtime>>::get(1).is_none()).unwrap());
		assert!(call_on_exposure!(<Previous<Runtime>>::get(2).is_some()).unwrap());
		assert!(call_on_exposure!(<Current<Runtime>>::get(0).is_some()).unwrap());
		assert!(call_on_exposure!(<Current<Runtime>>::get(1).is_none()).unwrap());
		assert!(call_on_exposure!(<Current<Runtime>>::get(2).is_none()).unwrap());
		assert!(call_on_exposure!(<Next<Runtime>>::get(0).is_none()).unwrap());
		assert!(call_on_exposure!(<Next<Runtime>>::get(1).is_some()).unwrap());
		assert!(call_on_exposure!(<Next<Runtime>>::get(2).is_none()).unwrap());
		assert_eq!(
			<ExposureCacheStates<Runtime>>::get(),
			(ExposureCacheState::Current, ExposureCacheState::Next, ExposureCacheState::Previous)
		);

		Staking::shift_exposure_cache_states();

		assert!(call_on_exposure!(<Previous<Runtime>>::get(0).is_some()).unwrap());
		assert!(call_on_exposure!(<Previous<Runtime>>::get(1).is_none()).unwrap());
		assert!(call_on_exposure!(<Previous<Runtime>>::get(2).is_none()).unwrap());
		assert!(call_on_exposure!(<Current<Runtime>>::get(0).is_none()).unwrap());
		assert!(call_on_exposure!(<Current<Runtime>>::get(1).is_some()).unwrap());
		assert!(call_on_exposure!(<Current<Runtime>>::get(2).is_none()).unwrap());
		assert!(call_on_exposure!(<Next<Runtime>>::get(0).is_none()).unwrap());
		assert!(call_on_exposure!(<Next<Runtime>>::get(1).is_none()).unwrap());
		assert!(call_on_exposure!(<Next<Runtime>>::get(2).is_some()).unwrap());
		assert_eq!(
			<ExposureCacheStates<Runtime>>::get(),
			(ExposureCacheState::Previous, ExposureCacheState::Current, ExposureCacheState::Next)
		);
	});
}

#[test]
fn stake_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(System::account(1).consumers, 0);
		assert!(Staking::ledger_of(1).is_none());
		assert_eq!(Balances::free_balance(1), 1_000 * UNIT);
		assert_eq!(Assets::balance(0, 1), 1_000 * UNIT);

		// Stake 1 RING.
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), UNIT, 0, Vec::new()));
		assert_eq!(System::account(1).consumers, 1);
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger { staked_ring: UNIT, ..ZeroDefault::default() }
		);
		assert_eq!(Balances::free_balance(1), 999 * UNIT);

		// Stake 1 KTON.
		assert_eq!(System::account(1).consumers, 1);
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), 0, UNIT, Vec::new()));
		assert_eq!(Assets::balance(0, 1), 999 * UNIT);
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger { staked_ring: UNIT, staked_kton: UNIT, ..ZeroDefault::default() }
		);

		// Stake invalid deposit.
		assert_noop!(
			Staking::stake(RuntimeOrigin::signed(1), 0, 0, vec![0]),
			<DepositError<Runtime>>::DepositNotFound
		);

		// Stake 1 deposit.
		assert_eq!(System::account(1).consumers, 1);
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), 0, 0, vec![0]));
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				staked_ring: UNIT,
				staked_kton: UNIT,
				staked_deposits: BoundedVec::truncate_from(vec![0]),
				..ZeroDefault::default()
			}
		);

		// Stake 500 RING, 500 KTON and 2 deposits.
		assert_eq!(System::account(1).consumers, 2);
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), 200 * UNIT, 1));
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), 200 * UNIT, 1));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), 500 * UNIT, 500 * UNIT, vec![1, 2]));
		assert_eq!(Balances::free_balance(1), 98 * UNIT);
		assert_eq!(Assets::balance(0, 1), 499 * UNIT + 3_053_299_492_385_785);
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				staked_ring: 501 * UNIT,
				staked_kton: 501 * UNIT,
				staked_deposits: BoundedVec::truncate_from(vec![0, 1, 2]),
				..ZeroDefault::default()
			}
		);
	});
}

#[test]
fn unstake_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), 3 * UNIT, 3 * UNIT, vec![0, 1, 2]));
		assert_eq!(Balances::free_balance(1), 994 * UNIT);
		assert_eq!(Assets::balance(0, 1), 997 * UNIT + 22_842_639_593_907);
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				staked_ring: 3 * UNIT,
				staked_kton: 3 * UNIT,
				staked_deposits: BoundedVec::truncate_from(vec![0, 1, 2]),
				..ZeroDefault::default()
			}
		);

		// Unstake 1 RING.
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(1), UNIT, 0, Vec::new()));
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				staked_ring: 2 * UNIT,
				staked_kton: 3 * UNIT,
				staked_deposits: BoundedVec::truncate_from(vec![0, 1, 2]),
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 6)]),
				..ZeroDefault::default()
			}
		);

		// Unstake 1 KTON.
		Efflux::block(1);
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(1), 0, UNIT, Vec::new()));
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				staked_ring: 2 * UNIT,
				staked_kton: 2 * UNIT,
				staked_deposits: BoundedVec::truncate_from(vec![0, 1, 2]),
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 6)]),
				..ZeroDefault::default()
			}
		);

		// Unstake invalid deposit.
		assert_noop!(
			Staking::unstake(RuntimeOrigin::signed(1), 0, 0, vec![3]),
			<Error<Runtime>>::DepositNotFound
		);

		// Unstake 1 deposit.
		Efflux::block(1);
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(1), 0, 0, vec![1]));
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				staked_ring: 2 * UNIT,
				staked_kton: 2 * UNIT,
				staked_deposits: BoundedVec::truncate_from(vec![0, 2]),
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 6)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(1, 8)]),
				..ZeroDefault::default()
			}
		);

		// Unstake 2 RING, 2 KTON and 2 deposits.
		Efflux::block(1);
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(1), 2 * UNIT, 2 * UNIT, vec![0, 2]));
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 6), (2 * UNIT, 9)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(1, 8), (0, 9), (2, 9)]),
				..ZeroDefault::default()
			}
		);

		// Keep the stakes for at least `MinStakingDuration`.
		assert_eq!(Balances::free_balance(1), 994 * UNIT);
		assert_eq!(Assets::balance(0, 1), 1_000 * UNIT + 22_842_639_593_907);
	});
}

#[test]
fn restake_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), 3 * UNIT, 3 * UNIT, vec![0, 1, 2]));
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(1), UNIT, UNIT, vec![0, 1, 2]));
		Efflux::block(1);
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(1), UNIT, UNIT, Vec::new()));
		Efflux::block(1);
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(1), UNIT, UNIT, Vec::new()));
		assert_eq!(Balances::free_balance(1), 994 * UNIT);
		assert_eq!(Assets::balance(0, 1), 1_000 * UNIT + 22_842_639_593_907);
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 6), (UNIT, 7), (UNIT, 8)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(0, 6), (1, 6), (2, 6)]),
				..ZeroDefault::default()
			}
		);

		// Restake 1.5 RING.
		assert_ok!(Staking::restake(RuntimeOrigin::signed(1), 3 * UNIT / 2, Vec::new()));
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				staked_ring: 3 * UNIT / 2,
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 6), (UNIT / 2, 7)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(0, 6), (1, 6), (2, 6)]),
				..ZeroDefault::default()
			}
		);

		// Restake invalid deposit.
		assert_noop!(
			Staking::unstake(RuntimeOrigin::signed(1), 0, 0, vec![3]),
			<Error<Runtime>>::DepositNotFound
		);

		// Restake 1 deposit.
		assert_ok!(Staking::restake(RuntimeOrigin::signed(1), 0, vec![1]));
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				staked_ring: 3 * UNIT / 2,
				staked_deposits: BoundedVec::truncate_from(vec![1]),
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 6), (UNIT / 2, 7)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(0, 6), (2, 6)]),
				..ZeroDefault::default()
			}
		);

		// Restake 1.5 RING and 2 deposits.
		Efflux::block(1);
		assert_ok!(Staking::restake(RuntimeOrigin::signed(1), 3 * UNIT / 2, vec![0, 2]));
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				staked_ring: 3 * UNIT,
				staked_deposits: BoundedVec::truncate_from(vec![1, 0, 2]),
				..ZeroDefault::default()
			}
		);
	});
}

#[test]
fn claim_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), 2 * UNIT, 2 * UNIT, vec![0, 1, 2]));
		assert_eq!(System::account(1).consumers, 2);
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(1), UNIT, 0, Vec::new()));
		Efflux::block(1);
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(1), 0, UNIT, Vec::new()));
		Efflux::block(1);
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(1), 0, 0, vec![0]));
		Efflux::block(1);
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(1), UNIT, UNIT, vec![1, 2]));
		assert_eq!(Balances::free_balance(1), 995 * UNIT);
		assert_eq!(Assets::balance(0, 1), 1_000 * UNIT + 22_842_639_593_907);
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 6), (UNIT, 9)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(0, 8), (1, 9), (2, 9)]),
				..ZeroDefault::default()
			}
		);

		// 4 expired.
		assert_ok!(Staking::claim(RuntimeOrigin::signed(1)));
		assert_eq!(System::account(1).consumers, 2);
		assert_eq!(Balances::free_balance(1), 996 * UNIT);
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 9)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(0, 8), (1, 9), (2, 9)]),
				..ZeroDefault::default()
			}
		);

		// 5 expired.
		Efflux::block(1);
		assert_ok!(Staking::claim(RuntimeOrigin::signed(1)));
		assert_eq!(System::account(1).consumers, 2);
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 9)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(0, 8), (1, 9), (2, 9)]),
				..ZeroDefault::default()
			}
		);

		// 6 expired.
		Efflux::block(1);
		assert_ok!(Staking::claim(RuntimeOrigin::signed(1)));
		assert_eq!(System::account(1).consumers, 2);
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 9)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(1, 9), (2, 9)]),
				..ZeroDefault::default()
			}
		);

		// 7 expired.
		Efflux::block(2);
		assert_ok!(Staking::claim(RuntimeOrigin::signed(1)));
		assert_eq!(System::account(1).consumers, 1);
		assert_eq!(Balances::free_balance(1), 997 * UNIT);
		assert!(Staking::ledger_of(1).is_none());
	});
}

#[test]
fn collect_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(Staking::collator_of(1).is_none());
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), UNIT, 0, Vec::new()));

		(0..=99).for_each(|c| {
			let c = Perbill::from_percent(c);

			assert_ok!(Staking::collect(RuntimeOrigin::signed(1), c));
			assert_eq!(Staking::collator_of(1).unwrap(), c);
		});
	});
}

#[test]
fn nominate_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), UNIT, 0, Vec::new()));
		assert_ok!(Staking::collect(RuntimeOrigin::signed(1), Perbill::zero()));

		(2..=10).for_each(|n| {
			assert!(Staking::nominator_of(n).is_none());
			assert_ok!(Staking::stake(RuntimeOrigin::signed(n), UNIT, 0, Vec::new()));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(n), 1));
			assert_eq!(Staking::nominator_of(n).unwrap(), 1);
		});
	});
}

#[test]
fn chill_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), UNIT, 0, Vec::new()));
		assert_ok!(Staking::collect(RuntimeOrigin::signed(1), Perbill::zero()));
		(2..=10).for_each(|n| {
			assert_ok!(Staking::stake(RuntimeOrigin::signed(n), UNIT, 0, Vec::new()));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(n), 1));
		});
		assert!(Staking::collator_of(1).is_some());
		(2..=10).for_each(|n| assert!(Staking::nominator_of(n).is_some()));

		(1..=10).for_each(|i| {
			assert_ok!(Staking::chill(RuntimeOrigin::signed(i)));
		});
		assert!(Staking::collator_of(1).is_none());
		(2..=10).for_each(|n| assert!(Staking::nominator_of(n).is_none()));
	});
}

#[test]
fn set_collator_count_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Staking::set_collator_count(RuntimeOrigin::signed(1), 1),
			DispatchError::BadOrigin
		);
		assert_noop!(
			Staking::set_collator_count(RuntimeOrigin::root(), 0),
			<Error<Runtime>>::ZeroCollatorCount
		);
		assert_ok!(Staking::set_collator_count(RuntimeOrigin::root(), 1));
	});
}

#[test]
fn power_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Staking::quick_power_of(&1), 0);
		assert_eq!(Staking::quick_power_of(&2), 0);
		assert_eq!(Staking::quick_power_of(&3), 0);
		assert_eq!(Staking::quick_power_of(&4), 0);

		// 1 stakes 1 RING.
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), UNIT, 0, Vec::new()));
		assert_eq!(Staking::quick_power_of(&1), 500_000_000);

		// 2 stakes 1 KTON.
		assert_ok!(Staking::stake(RuntimeOrigin::signed(2), 0, UNIT, Vec::new()));
		assert_eq!(Staking::quick_power_of(&1), 500_000_000);
		assert_eq!(Staking::quick_power_of(&2), 500_000_000);

		// 3 stakes 1 deposit.
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(3), UNIT, 1));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(3), 0, 0, vec![0]));
		assert_eq!(Staking::quick_power_of(&1), 250_000_000);
		assert_eq!(Staking::quick_power_of(&2), 500_000_000);
		assert_eq!(Staking::quick_power_of(&3), 250_000_000);

		// 4 stakes 1 KTON.
		assert_ok!(Staking::stake(RuntimeOrigin::signed(4), 0, UNIT, Vec::new()));
		assert_eq!(Staking::quick_power_of(&1), 250_000_000);
		assert_eq!(Staking::quick_power_of(&2), 250_000_000);
		assert_eq!(Staking::quick_power_of(&3), 250_000_000);
		assert_eq!(Staking::quick_power_of(&4), 250_000_000);

		// 1 unstakes 1 RING.
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(1), UNIT, 0, Vec::new()));
		assert_eq!(Staking::quick_power_of(&1), 0);
		assert_eq!(Staking::quick_power_of(&2), 250_000_000);
		assert_eq!(Staking::quick_power_of(&3), 500_000_000);
		assert_eq!(Staking::quick_power_of(&4), 250_000_000);

		// 2 unstakes 1 KTON.
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(2), 0, UNIT, Vec::new()));
		assert_eq!(Staking::quick_power_of(&1), 0);
		assert_eq!(Staking::quick_power_of(&2), 0);
		assert_eq!(Staking::quick_power_of(&3), 500_000_000);
		assert_eq!(Staking::quick_power_of(&4), 500_000_000);

		// 3 unstakes 1 deposit.
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(3), UNIT, 1));
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(3), 0, 0, vec![0]));
		assert_eq!(Staking::quick_power_of(&1), 0);
		assert_eq!(Staking::quick_power_of(&2), 0);
		assert_eq!(Staking::quick_power_of(&3), 0);
		assert_eq!(Staking::quick_power_of(&4), 500_000_000);

		// 4 unstakes 1 KTON.
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(4), 0, UNIT, Vec::new()));
		assert_eq!(Staking::quick_power_of(&1), 0);
		assert_eq!(Staking::quick_power_of(&2), 0);
		assert_eq!(Staking::quick_power_of(&3), 0);
		assert_eq!(Staking::quick_power_of(&4), 0);
	});
}

#[test]
fn elect_should_work() {
	ExtBuilder::default().collator_count(3).build().execute_with(|| {
		(1..=5).for_each(|i| {
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(i),
				i as Balance * UNIT,
				UNIT,
				Vec::new()
			));
			assert_ok!(Staking::collect(RuntimeOrigin::signed(i), Perbill::zero()));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(i), i));
		});
		(6..=10).for_each(|i| {
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(i),
				0,
				(11 - i as Balance) * UNIT,
				Vec::new()
			));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(i), i - 5));
		});

		assert_eq!(Staking::elect().unwrap(), vec![5, 4, 3]);
	});
	ExtBuilder::default().collator_count(3).build().execute_with(|| {
		(1..=5).for_each(|i| {
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(i),
				i as Balance * UNIT,
				0,
				Vec::new()
			));
			assert_ok!(Staking::collect(RuntimeOrigin::signed(i), Perbill::zero()));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(i), i));
		});
		(6..=10).for_each(|i| {
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(i),
				UNIT,
				(11 - i as Balance) * UNIT,
				Vec::new()
			));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(i), i - 5));
		});

		assert_eq!(Staking::elect().unwrap(), vec![1, 2, 3]);
	});
}

#[test]
fn payout_should_work() {
	ExtBuilder::default().collator_count(5).build().execute_with(|| {
		(1..=5).for_each(|i| {
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(i),
				0,
				i as Balance * UNIT,
				Vec::new()
			));
			assert_ok!(Staking::collect(RuntimeOrigin::signed(i), Perbill::from_percent(i * 10)));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(i), i));
		});
		(6..=10).for_each(|i| {
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(i),
				0,
				(11 - i as Balance) * UNIT,
				Vec::new()
			));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(i), i - 5));
		});
		new_session();
		(1..=10).for_each(|i| assert_eq!(Balances::free_balance(i), 1_000 * UNIT));

		let session_duration = Duration::new(12 * 600, 0).as_millis();
		Efflux::time(session_duration - <Period as Get<u64>>::get() as Moment);
		Staking::note_authors(&[1, 2, 3, 4, 5]);
		new_session();
		new_session();
		payout();

		let rewards = vec![
			364298724080145719490,
			680024276867030965392,
			947176684881602914390,
			1165755919271402550091,
			1335761993442622950820,
			1092896174426229508197,
			777170622950819672131,
			510018214936247723133,
			291438979672131147541,
			121432905646630236794,
		];
		assert_eq!(
			rewards,
			(1..=10).map(|i| Balances::free_balance(i) - 1_000 * UNIT).collect::<Vec<_>>()
		);
		assert_eq_error_rate!(
			PayoutFraction::get()
				* dc_inflation::issuing_in_period(session_duration, Timestamp::now()).unwrap(),
			rewards.iter().sum::<Balance>(),
			// Error rate 1 RING.
			UNIT
		);
	});

	ExtBuilder::default().inflation_type(1).collator_count(5).build().execute_with(|| {
		(1..=5).for_each(|i| {
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(i),
				0,
				i as Balance * UNIT,
				Vec::new()
			));
			assert_ok!(Staking::collect(RuntimeOrigin::signed(i), Perbill::from_percent(i * 10)));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(i), i));
		});
		(6..=10).for_each(|i| {
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(i),
				0,
				(11 - i as Balance) * UNIT,
				Vec::new()
			));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(i), i - 5));
		});
		new_session();
		new_session();
		(1..=10).for_each(|i| assert_eq!(Balances::free_balance(i), 1_000 * UNIT));

		let total_issuance = Balances::total_issuance();
		let session_duration = Duration::new(12 * 600, 0).as_millis();
		Efflux::time(session_duration - <Period as Get<u64>>::get() as Moment);
		Staking::note_authors(&[1, 2, 3, 4, 5]);
		new_session();
		payout();

		let rewards = vec![
			499999998800000000000,
			933333320000000000000,
			1300000000000000000000,
			1599999999200000000000,
			1833333336000000000000,
			1499999999400000000000,
			1066666680000000000000,
			700000000000000000000,
			399999999600000000000,
			166666663000000000000,
		];
		assert_eq!(
			rewards,
			(1..=10).map(|i| Balances::free_balance(i) - 1_000 * UNIT).collect::<Vec<_>>()
		);

		assert_eq!(Balances::total_issuance(), total_issuance);
		assert_eq!(
			Balances::free_balance(&Treasury::account_id()),
			1_000_000 * UNIT - rewards.iter().sum::<Balance>()
		);

		assert_ok!(Balances::transfer_all(
			RuntimeOrigin::signed(Treasury::account_id()),
			Default::default(),
			false
		));
		Staking::note_authors(&[1]);
		System::reset_events();
		new_session();
		payout();

		assert_eq!(
			System::events()
				.into_iter()
				.filter_map(|e| match e.event {
					RuntimeEvent::Staking(e @ Event::Unpaid { .. }) => Some(e),
					_ => None,
				})
				.collect::<Vec<_>>(),
			vec![
				Event::Unpaid { staker: 6, amount: 7499999997000000000000 },
				Event::Unpaid { staker: 1, amount: 2499999994000000000000 }
			]
		);
	});
}

#[test]
fn auto_payout_should_work() {
	ExtBuilder::default().collator_count(2).build().execute_with(|| {
		(1..=2).for_each(|i| {
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(i),
				0,
				i as Balance * UNIT,
				Vec::new()
			));
			assert_ok!(Staking::collect(RuntimeOrigin::signed(i), Perbill::from_percent(i * 10)));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(i), i));
		});
		(3..=4).for_each(|i| {
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(i),
				0,
				(11 - i as Balance) * UNIT,
				Vec::new()
			));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(i), i - 2));
		});
		new_session();
		new_session();

		Efflux::time(<Period as Get<u64>>::get() as Moment);
		Staking::note_authors(&[1, 2]);
		new_session();
		(1..=4).for_each(|i| assert_eq!(Balances::free_balance(i), 1_000 * UNIT));

		Efflux::block(1);
		assert_eq!(Balances::free_balance(1), 1000000607164541287188);
		assert_eq!(Balances::free_balance(2), 1000000000000000000000);
		assert_eq!(Balances::free_balance(3), 1000002428658163934426);
		assert_eq!(Balances::free_balance(4), 1000000000000000000000);

		Efflux::block(1);
		assert_eq!(Balances::free_balance(1), 1000000607164541287188);
		assert_eq!(Balances::free_balance(2), 1000001146866363084396);
		assert_eq!(Balances::free_balance(3), 1000002428658163934426);
		assert_eq!(Balances::free_balance(4), 1000001888956344869459);

		Efflux::block(1);
		assert_eq!(Balances::free_balance(1), 1000000607164541287188);
		assert_eq!(Balances::free_balance(2), 1000001146866363084396);
		assert_eq!(Balances::free_balance(3), 1000002428658163934426);
		assert_eq!(Balances::free_balance(4), 1000001888956344869459);
	});
}

#[test]
fn on_new_session_should_work() {
	ExtBuilder::default().collator_count(2).genesis_collator().build().execute_with(|| {
		assert_eq_uvec!(
			darwinia_staking::call_on_exposure!(
				<Previous<Runtime>>::iter_keys().collect::<Vec<_>>()
			)
			.unwrap(),
			[1, 2]
		);
		assert_eq_uvec!(
			darwinia_staking::call_on_exposure!(<Current<Runtime>>::iter_keys().collect::<Vec<_>>()).unwrap(),
			[1, 2]
		);
		assert_eq_uvec!(
			darwinia_staking::call_on_exposure!(<Next<Runtime>>::iter_keys().collect::<Vec<_>>())
				.unwrap(),
			[1, 2]
		);

		assert_ok!(Staking::collect(RuntimeOrigin::signed(3), Perbill::zero()));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(3), 2 * UNIT, 0, Vec::new()));
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(3), 3));
		Staking::note_authors(&Session::validators());

		new_session();
		assert_eq_uvec!(
			darwinia_staking::call_on_exposure!(
				<Previous<Runtime>>::iter_keys().collect::<Vec<_>>()
			)
			.unwrap(),
			[1, 2]
		);
		assert_eq_uvec!(
			darwinia_staking::call_on_exposure!(<Current<Runtime>>::iter_keys().collect::<Vec<_>>()).unwrap(),
			[1, 2]
		);
		assert_eq_uvec!(
			darwinia_staking::call_on_exposure!(<Next<Runtime>>::iter_keys().collect::<Vec<_>>())
				.unwrap(),
			[1, 3]
		);

		assert_ok!(Staking::chill(RuntimeOrigin::signed(3)));
		assert_ok!(Staking::collect(RuntimeOrigin::signed(4), Perbill::zero()));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(4), 2 * UNIT, 0, Vec::new()));
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(4), 4));
		Staking::note_authors(&Session::validators());

		new_session();
		assert_eq_uvec!(
			darwinia_staking::call_on_exposure!(
				<Previous<Runtime>>::iter_keys().collect::<Vec<_>>()
			)
			.unwrap(),
			[1, 2]
		);
		assert_eq_uvec!(
			darwinia_staking::call_on_exposure!(<Current<Runtime>>::iter_keys().collect::<Vec<_>>()).unwrap(),
			[1, 3]
		);
		assert_eq_uvec!(
			darwinia_staking::call_on_exposure!(<Next<Runtime>>::iter_keys().collect::<Vec<_>>())
				.unwrap(),
			[1, 4]
		);

		assert_ok!(Staking::chill(RuntimeOrigin::signed(4)));
		assert_ok!(Staking::collect(RuntimeOrigin::signed(5), Perbill::zero()));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(5), 2 * UNIT, 0, Vec::new()));
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(5), 5));
		Staking::note_authors(&Session::validators());

		new_session();
		assert_eq_uvec!(
			darwinia_staking::call_on_exposure!(
				<Previous<Runtime>>::iter_keys().collect::<Vec<_>>()
			)
			.unwrap(),
			[1, 3]
		);
		assert_eq_uvec!(
			darwinia_staking::call_on_exposure!(<Current<Runtime>>::iter_keys().collect::<Vec<_>>()).unwrap(),
			[1, 4]
		);
		assert_eq_uvec!(
			darwinia_staking::call_on_exposure!(<Next<Runtime>>::iter_keys().collect::<Vec<_>>())
				.unwrap(),
			[1, 5]
		);
	});
}

#[test]
fn migration_curves_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(10);
		<MigrationStartBlock<Runtime>>::put(10);

		assert_eq!(
			vec![0, 1, 7, 14, 21, 29, 30, 31, 999]
				.into_iter()
				.map(|x| {
					System::set_block_number(10 + x * 24 * 60 * 60 / 12);

					let x = <MigrationCurve<Runtime>>::get();
					let y = migration_curve_kton_reward(x);

					format!("{x:?} -> {y:?}")
				})
				.collect::<Vec<_>>(),
			[
				"99.9995370370370371% -> 49.9998842589913402%",
				"96.6666666666666667% -> 49.1525423728813559%",
				"76.6666666666666667% -> 43.3962264150943396%",
				"53.3333333333333334% -> 34.7826086956521739%",
				"30% -> 23.0769230769230769%",
				"3.3333333333333334% -> 3.2258064516129032%",
				"0% -> 0%",
				"0% -> 0%",
				"0% -> 0%"
			]
		);
	});
}

#[test]
fn test_notify_mocked_signature_alway_valid() {
	assert!(TransactionSignature::new(
		38,
		array_bytes::hex_n_into_unchecked::<_, _, 32>(
			"be67e0a07db67da8d446f76add590e54b6e92cb6b8f9835aeb67540579a27717",
		),
		array_bytes::hex_n_into_unchecked::<_, _, 32>(
			"2d690516512020171c1ec870f6ff45398cc8609250326be89915fb538e7bd718",
		),
	)
	.is_some());
}
