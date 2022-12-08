// This file is part of Darwinia.
//
// Copyright (C) 2018-2022 Darwinia Network
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

mod mock;
use mock::*;

// core
use core::time::Duration;
// darwinia
use darwinia_staking::*;
use dc_types::{Balance, UNIT};
// substrate
use frame_support::{assert_ok, BoundedVec};
use sp_runtime::{assert_eq_error_rate, Perbill};

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
			Ledger { account: 1, staked_ring: UNIT, ..ZeroDefault::default() }
		);
		assert_eq!(Balances::free_balance(1), 999 * UNIT);

		// Stake 1 KTON.
		assert_eq!(System::account(1).consumers, 1);
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), 0, UNIT, Vec::new()));
		assert_eq!(Assets::balance(0, 1), 999 * UNIT);
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger { account: 1, staked_ring: UNIT, staked_kton: UNIT, ..ZeroDefault::default() }
		);

		// Stake 1 deposit.
		assert_eq!(System::account(1).consumers, 1);
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), 0, 0, vec![0]));
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				account: 1,
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
				account: 1,
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
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), 2 * UNIT, 2 * UNIT, vec![0, 1, 2]));
		assert_eq!(Balances::free_balance(1), 995 * UNIT);
		assert_eq!(Assets::balance(0, 1), 998 * UNIT + 22_842_639_593_907);
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				account: 1,
				staked_ring: 2 * UNIT,
				staked_kton: 2 * UNIT,
				staked_deposits: BoundedVec::truncate_from(vec![0, 1, 2]),
				..ZeroDefault::default()
			}
		);

		// Unstake 1 RING.
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(1), UNIT, 0, Vec::new()));
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				account: 1,
				staked_ring: 1 * UNIT,
				staked_kton: 2 * UNIT,
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
				account: 1,
				staked_ring: 1 * UNIT,
				staked_kton: 1 * UNIT,
				staked_deposits: BoundedVec::truncate_from(vec![0, 1, 2]),
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 4)]),
				unstaking_kton: BoundedVec::truncate_from(vec![(UNIT, 5)]),
				..ZeroDefault::default()
			}
		);

		// Unstake 1 deposit.
		Efflux::block(1);
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(1), 0, 0, vec![0]));
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				account: 1,
				staked_ring: 1 * UNIT,
				staked_kton: 1 * UNIT,
				staked_deposits: BoundedVec::truncate_from(vec![1, 2]),
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 4)]),
				unstaking_kton: BoundedVec::truncate_from(vec![(UNIT, 5)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(0, 6)])
			}
		);

		// Unstake 1 RING, 1 KTON and 2 deposits.
		Efflux::block(1);
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(1), UNIT, UNIT, vec![1, 2]));
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				account: 1,
				staked_ring: 0,
				staked_kton: 0,
				staked_deposits: Default::default(),
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 4), (UNIT, 7)]),
				unstaking_kton: BoundedVec::truncate_from(vec![(UNIT, 5), (UNIT, 7)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(0, 6), (1, 7), (2, 7)])
			}
		);

		// Keep the stakes for at least `MinStakingDuration`.
		assert_eq!(Balances::free_balance(1), 995 * UNIT);
		assert_eq!(Assets::balance(0, 1), 998 * UNIT + 22_842_639_593_907);
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
				account: 1,
				staked_ring: 0,
				staked_kton: 0,
				staked_deposits: Default::default(),
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 4), (UNIT, 7)]),
				unstaking_kton: BoundedVec::truncate_from(vec![(UNIT, 5), (UNIT, 7)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(0, 6), (1, 7), (2, 7)])
			}
		);

		// 4 expired.
		assert_ok!(Staking::claim(RuntimeOrigin::signed(1)));
		assert_eq!(System::account(1).consumers, 2);
		assert_eq!(Balances::free_balance(1), 996 * UNIT);
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger {
				account: 1,
				staked_ring: 0,
				staked_kton: 0,
				staked_deposits: Default::default(),
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 7)]),
				unstaking_kton: BoundedVec::truncate_from(vec![(UNIT, 5), (UNIT, 7)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(0, 6), (1, 7), (2, 7)])
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
				account: 1,
				staked_ring: 0,
				staked_kton: 0,
				staked_deposits: Default::default(),
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 7)]),
				unstaking_kton: BoundedVec::truncate_from(vec![(UNIT, 7)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(0, 6), (1, 7), (2, 7)])
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
				account: 1,
				staked_ring: 0,
				staked_kton: 0,
				staked_deposits: Default::default(),
				unstaking_ring: BoundedVec::truncate_from(vec![(UNIT, 7)]),
				unstaking_kton: BoundedVec::truncate_from(vec![(UNIT, 7)]),
				unstaking_deposits: BoundedVec::truncate_from(vec![(1, 7), (2, 7)])
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

		(0..=100).for_each(|c| {
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
		assert_ok!(Staking::collect(RuntimeOrigin::signed(1), Default::default()));

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
		assert_ok!(Staking::collect(RuntimeOrigin::signed(1), Default::default()));
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
			assert_ok!(Staking::collect(RuntimeOrigin::signed(i), Default::default()));
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
			assert_ok!(Staking::collect(RuntimeOrigin::signed(i), Default::default()));
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
		Staking::elect();
		Staking::reward_by_ids(&[(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)]);
		(1..=10).for_each(|i| assert_eq!(Balances::free_balance(i), 1_000 * UNIT));

		let session_duration = Duration::new(6 * 60 * 60, 0).as_millis();
		Staking::payout(session_duration, Staking::elapsed_time());
		let rewards = [
			683_059_435_062_369_982_561_u128,
			1_275_044_256_196_592_252_422,
			1_775_954_509_850_707_836_449,
			2_185_790_171_434_577_367_478,
			2_504_551_227_287_012_308_196,
			2_049_178_272_400_257_458_130,
			1_457_193_451_266_035_188_269,
			956_283_197_611_919_604_242,
			546_447_536_028_050_073_213,
			227_686_480_175_615_132_495,
		];
		(1..=10)
			.zip(rewards.iter())
			.for_each(|(i, r)| assert_eq!(Balances::free_balance(i), 1_000 * UNIT + r));
		assert_eq_error_rate!(
			<Runtime as darwinia_staking::Config>::PayoutFraction::get()
				* dc_inflation::in_period(
					dc_inflation::TOTAL_SUPPLY - Balances::total_issuance(),
					session_duration,
					0
				)
				.unwrap(),
			rewards.iter().sum::<Balance>(),
			// Error rate 0.02 RING
			2 * UNIT / 100
		);
	});
}
