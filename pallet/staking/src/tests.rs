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
// darwinia
use crate::{mock::*, *};
use darwinia_deposit::Error as DepositError;
use dc_types::{Balance, UNIT};
// substrate
use frame_support::{assert_noop, assert_ok, BoundedVec};
use sp_runtime::{assert_eq_error_rate, DispatchError, Perbill};
use substrate_test_utils::assert_eq_uvec;

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
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 4)]),
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
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 4)]),
				unstaking_kton: BoundedVec::truncate_from(vec![(UNIT, 5)]),
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
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 4)]),
				unstaking_kton: BoundedVec::truncate_from(vec![(UNIT, 5)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(1, 6)])
			}
		);

		// Unstake 2 RING, 2 KTON and 2 deposits.
		Efflux::block(1);
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(1), 2 * UNIT, 2 * UNIT, vec![0, 2]));
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 4), (2 * UNIT, 7)]),
				unstaking_kton: BoundedVec::truncate_from(vec![(UNIT, 5), (2 * UNIT, 7)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(1, 6), (0, 7), (2, 7)]),
				..ZeroDefault::default()
			}
		);

		// Keep the stakes for at least `MinStakingDuration`.
		assert_eq!(Balances::free_balance(1), 994 * UNIT);
		assert_eq!(Assets::balance(0, 1), 997 * UNIT + 22_842_639_593_907);
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
		assert_eq!(Assets::balance(0, 1), 997 * UNIT + 22_842_639_593_907);
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 4), (UNIT, 5), (UNIT, 6)]),
				unstaking_kton: BoundedVec::truncate_from(vec![(UNIT, 4), (UNIT, 5), (UNIT, 6)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(0, 4), (1, 4), (2, 4)]),
				..ZeroDefault::default()
			}
		);

		// Restake 1.5 RING.
		assert_ok!(Staking::restake(RuntimeOrigin::signed(1), 3 * UNIT / 2, 0, Vec::new()));
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				staked_ring: 3 * UNIT / 2,
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 4), (UNIT / 2, 5)]),
				unstaking_kton: BoundedVec::truncate_from(vec![(UNIT, 4), (UNIT, 5), (UNIT, 6)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(0, 4), (1, 4), (2, 4)]),
				..ZeroDefault::default()
			}
		);

		// Restake 1.5 KTON.
		assert_ok!(Staking::restake(RuntimeOrigin::signed(1), 0, 3 * UNIT / 2, Vec::new()));
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				staked_ring: 3 * UNIT / 2,
				staked_kton: 3 * UNIT / 2,
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 4), (UNIT / 2, 5)]),
				unstaking_kton: BoundedVec::truncate_from(vec![(UNIT, 4), (UNIT / 2, 5)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(0, 4), (1, 4), (2, 4)]),
				..ZeroDefault::default()
			}
		);

		// Restake invalid deposit.
		assert_noop!(
			Staking::unstake(RuntimeOrigin::signed(1), 0, 0, vec![3]),
			<Error<Runtime>>::DepositNotFound
		);

		// Restake 1 deposit.
		assert_ok!(Staking::restake(RuntimeOrigin::signed(1), 0, 0, vec![1]));
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				staked_ring: 3 * UNIT / 2,
				staked_kton: 3 * UNIT / 2,
				staked_deposits: BoundedVec::truncate_from(vec![1]),
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 4), (UNIT / 2, 5)]),
				unstaking_kton: BoundedVec::truncate_from(vec![(UNIT, 4), (UNIT / 2, 5)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(0, 4), (2, 4)]),
			}
		);

		// Restake 1.5 RING, 1.5 KTON and 2 deposits.
		Efflux::block(1);
		assert_ok!(Staking::restake(
			RuntimeOrigin::signed(1),
			3 * UNIT / 2,
			3 * UNIT / 2,
			vec![0, 2]
		));
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				staked_ring: 3 * UNIT,
				staked_kton: 3 * UNIT,
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
		assert_eq!(Assets::balance(0, 1), 998 * UNIT + 22_842_639_593_907);
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 4), (UNIT, 7)]),
				unstaking_kton: BoundedVec::truncate_from(vec![(UNIT, 5), (UNIT, 7)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(0, 6), (1, 7), (2, 7)]),
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
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 7)]),
				unstaking_kton: BoundedVec::truncate_from(vec![(UNIT, 5), (UNIT, 7)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(0, 6), (1, 7), (2, 7)]),
				..ZeroDefault::default()
			}
		);

		// 5 expired.
		Efflux::block(1);
		assert_ok!(Staking::claim(RuntimeOrigin::signed(1)));
		assert_eq!(System::account(1).consumers, 2);
		assert_eq!(Assets::balance(0, 1), 999 * UNIT + 22_842_639_593_907);
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 7)]),
				unstaking_kton: BoundedVec::truncate_from(vec![(UNIT, 7)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(0, 6), (1, 7), (2, 7)]),
				..ZeroDefault::default()
			}
		);

		// 6 expired.
		Efflux::block(1);
		assert_ok!(Staking::claim(RuntimeOrigin::signed(1)));
		assert_eq!(System::account(1).consumers, 2);
		assert_eq!(Assets::balance(0, 1), 999 * UNIT + 22_842_639_593_907);
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 7)]),
				unstaking_kton: BoundedVec::truncate_from(vec![(UNIT, 7)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(1, 7), (2, 7)]),
				..ZeroDefault::default()
			}
		);

		// 7 expired.
		Efflux::block(2);
		assert_ok!(Staking::claim(RuntimeOrigin::signed(1)));
		assert_eq!(System::account(1).consumers, 1);
		assert_eq!(Balances::free_balance(1), 997 * UNIT);
		assert_eq!(Assets::balance(0, 1), 1_000 * UNIT + 22_842_639_593_907);
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
		assert_eq!(Staking::power_of(&1), 0);
		assert_eq!(Staking::power_of(&2), 0);
		assert_eq!(Staking::power_of(&3), 0);
		assert_eq!(Staking::power_of(&4), 0);

		// 1 stakes 1 RING.
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), UNIT, 0, Vec::new()));
		assert_eq!(Staking::power_of(&1), 500_000_000);

		// 2 stakes 1 KTON.
		assert_ok!(Staking::stake(RuntimeOrigin::signed(2), 0, UNIT, Vec::new()));
		assert_eq!(Staking::power_of(&1), 500_000_000);
		assert_eq!(Staking::power_of(&2), 500_000_000);

		// 3 stakes 1 deposit.
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(3), UNIT, 1));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(3), 0, 0, vec![0]));
		assert_eq!(Staking::power_of(&1), 250_000_000);
		assert_eq!(Staking::power_of(&2), 500_000_000);
		assert_eq!(Staking::power_of(&3), 250_000_000);

		// 4 stakes 1 KTON.
		assert_ok!(Staking::stake(RuntimeOrigin::signed(4), 0, UNIT, Vec::new()));
		assert_eq!(Staking::power_of(&1), 250_000_000);
		assert_eq!(Staking::power_of(&2), 250_000_000);
		assert_eq!(Staking::power_of(&3), 250_000_000);
		assert_eq!(Staking::power_of(&4), 250_000_000);

		// 1 unstakes 1 RING.
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(1), UNIT, 0, Vec::new()));
		assert_eq!(Staking::power_of(&1), 0);
		assert_eq!(Staking::power_of(&2), 250_000_000);
		assert_eq!(Staking::power_of(&3), 500_000_000);
		assert_eq!(Staking::power_of(&4), 250_000_000);

		// 2 unstakes 1 KTON.
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(2), 0, UNIT, Vec::new()));
		assert_eq!(Staking::power_of(&1), 0);
		assert_eq!(Staking::power_of(&2), 0);
		assert_eq!(Staking::power_of(&3), 500_000_000);
		assert_eq!(Staking::power_of(&4), 500_000_000);

		// 3 unstakes 1 deposit.
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(3), UNIT, 1));
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(3), 0, 0, vec![0]));
		assert_eq!(Staking::power_of(&1), 0);
		assert_eq!(Staking::power_of(&2), 0);
		assert_eq!(Staking::power_of(&3), 0);
		assert_eq!(Staking::power_of(&4), 500_000_000);

		// 4 unstakes 1 KTON.
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(4), 0, UNIT, Vec::new()));
		assert_eq!(Staking::power_of(&1), 0);
		assert_eq!(Staking::power_of(&2), 0);
		assert_eq!(Staking::power_of(&3), 0);
		assert_eq!(Staking::power_of(&4), 0);
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

		assert_eq!(Staking::elect(), vec![5, 4, 3]);
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

		assert_eq!(Staking::elect(), vec![1, 2, 3]);
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
		new_session();
		Staking::reward_by_ids(&[(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)]);
		(1..=10).for_each(|i| assert_eq!(Balances::free_balance(i), 1_000 * UNIT));

		let session_duration = Duration::new(6 * 60 * 60, 0).as_millis();
		Timestamp::set_timestamp(session_duration);
		dbg!(Staking::elapsed_time());
		dbg!(Timestamp::now());
		OnDarwiniaSessionEnd::on_session_end();
		// Staking::payout(session_duration, Staking::elapsed_time());
		let rewards = [
			1_366_118_850_452_628_471_390,
			2_550_088_490_535_282_845_143,
			3_551_909_019_701_415_672_898,
			4_371_580_329_754_413_739_136,
			5_009_102_470_967_450_861_167,
			4_098_356_559_554_598_536_559,
			2_914_386_924_389_972_036_239,
			1_912_566_395_223_839_208_483,
			1_092_895_081_892_155_893_291,
			455_372_941_225_566_312_752,
		];
		(1..=10)
			.zip(rewards.iter())
			.for_each(|(i, r)| assert_eq!(Balances::free_balance(i), 1_000 * UNIT + r));
		assert_eq_error_rate!(
			PayoutFraction::get()
				* dc_inflation::in_period(
					dc_inflation::TOTAL_SUPPLY - Balances::total_issuance(),
					session_duration,
					0
				)
				.unwrap(),
			rewards.iter().sum::<Balance>(),
			// Error rate 0.1 RING.
			UNIT / 10
		);
	});
}

#[test]
fn on_new_session_should_work() {
	ExtBuilder::default().collator_count(2).genesis_collator().build().execute_with(|| {
		assert_eq_uvec!(<Exposures<Runtime>>::iter_keys().collect::<Vec<_>>(), [1, 2]);
		assert_eq_uvec!(<NextExposures<Runtime>>::iter_keys().collect::<Vec<_>>(), [1, 2]);

		assert_ok!(Staking::collect(RuntimeOrigin::signed(3), Perbill::zero()));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(3), 2 * UNIT, 0, Vec::new()));
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(3), 3));
		Staking::reward_by_ids(
			&Session::validators().into_iter().map(|v| (v, 20)).collect::<Vec<_>>(),
		);

		new_session();
		assert_eq_uvec!(<Exposures<Runtime>>::iter_keys().collect::<Vec<_>>(), [1, 2]);
		assert_eq_uvec!(<NextExposures<Runtime>>::iter_keys().collect::<Vec<_>>(), [1, 3]);

		assert_ok!(Staking::chill(RuntimeOrigin::signed(3)));
		assert_ok!(Staking::collect(RuntimeOrigin::signed(4), Perbill::zero()));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(4), 2 * UNIT, 0, Vec::new()));
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(4), 4));
		Staking::reward_by_ids(
			&Session::validators().into_iter().map(|v| (v, 20)).collect::<Vec<_>>(),
		);

		new_session();
		assert_eq_uvec!(<Exposures<Runtime>>::iter_keys().collect::<Vec<_>>(), [1, 3]);
		assert_eq_uvec!(<NextExposures<Runtime>>::iter_keys().collect::<Vec<_>>(), [1, 4]);

		assert_ok!(Staking::chill(RuntimeOrigin::signed(4)));
		assert_ok!(Staking::collect(RuntimeOrigin::signed(5), Perbill::zero()));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(5), 2 * UNIT, 0, Vec::new()));
		assert_ok!(Staking::nominate(RuntimeOrigin::signed(5), 5));
		Staking::reward_by_ids(
			&Session::validators().into_iter().map(|v| (v, 20)).collect::<Vec<_>>(),
		);

		new_session();
		assert_eq_uvec!(<Exposures<Runtime>>::iter_keys().collect::<Vec<_>>(), [1, 4]);
		assert_eq_uvec!(<NextExposures<Runtime>>::iter_keys().collect::<Vec<_>>(), [1, 5]);
	});
}
