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

// core
use core::time::Duration;
// darwinia
use crate::{mock::*, *};
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
		assert_eq!(Staking::rate_limit_state(), RateLimiter::Pos(0));

		// Stake 1 RING.
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), UNIT, Vec::new()));
		assert_eq!(System::account(1).consumers, 1);
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger { ring: UNIT, deposits: Default::default() }
		);
		assert_eq!(Balances::free_balance(1), 999 * UNIT);
		assert_eq!(Staking::rate_limit_state(), RateLimiter::Pos(UNIT));

		// Stake invalid deposit.
		assert_noop!(
			Staking::stake(RuntimeOrigin::signed(1), 0, vec![0]),
			<DepositError<Runtime>>::DepositNotFound
		);

		// Stake 1 deposit.
		assert_eq!(System::account(1).consumers, 1);
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), 0, vec![0]));
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger { ring: UNIT, deposits: BoundedVec::truncate_from(vec![0]) }
		);
		assert_eq!(Staking::rate_limit_state(), RateLimiter::Pos(2 * UNIT));

		// Stake 2 RING and 2 deposits.
		assert_eq!(System::account(1).consumers, 2);
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), 2 * UNIT, vec![1, 2]));
		assert_eq!(Balances::free_balance(1), 994 * UNIT);
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger { ring: 3 * UNIT, deposits: BoundedVec::truncate_from(vec![0, 1, 2]) }
		);
		assert_eq!(Staking::rate_limit_state(), RateLimiter::Pos(6 * UNIT));
	});
}

#[test]
fn unstake_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), UNIT, 1));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), 3 * UNIT, vec![0, 1, 2]));
		assert_eq!(Balances::free_balance(1), 994 * UNIT);
		assert_eq!(Staking::rate_limit_state(), RateLimiter::Pos(6 * UNIT));
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger { ring: 3 * UNIT, deposits: BoundedVec::truncate_from(vec![0, 1, 2]) }
		);
		assert_eq!(
			Deposit::deposit_of(1).unwrap().into_iter().map(|d| d.in_use).collect::<Vec<_>>(),
			[true, true, true]
		);

		// Unstake 1 RING.
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(1), UNIT, Vec::new()));
		assert_eq!(Staking::rate_limit_state(), RateLimiter::Pos(5 * UNIT));
		assert_eq!(Balances::free_balance(1), 995 * UNIT);
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger { ring: 2 * UNIT, deposits: BoundedVec::truncate_from(vec![0, 1, 2]) }
		);

		// Unstake invalid deposit.
		assert_noop!(
			Staking::unstake(RuntimeOrigin::signed(1), 0, vec![3]),
			<Error<Runtime>>::DepositNotFound
		);

		// Unstake 1 deposit.
		Efflux::block(1);
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(1), 0, vec![1]));
		assert_eq!(Staking::rate_limit_state(), RateLimiter::Pos(4 * UNIT));
		assert_eq!(
			Staking::ledger_of(1).unwrap(),
			Ledger { ring: 2 * UNIT, deposits: BoundedVec::truncate_from(vec![0, 2]) }
		);
		assert_eq!(
			Deposit::deposit_of(1).unwrap().into_iter().map(|d| d.in_use).collect::<Vec<_>>(),
			[true, false, true]
		);

		// Unstake 2 RING and 2 deposits.
		Efflux::block(1);
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(1), 2 * UNIT, vec![0, 2]));
		assert_eq!(Staking::rate_limit_state(), RateLimiter::Pos(0));
		assert!(Staking::ledger_of(1).is_none());
		assert_eq!(
			Deposit::deposit_of(1).unwrap().into_iter().map(|d| d.in_use).collect::<Vec<_>>(),
			[false, false, false]
		);

		// Prepare rate limit test data.
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(1), 100 * UNIT + 1, 1));
		<RateLimit<Runtime>>::put(200 * UNIT + 2);
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), 100 * UNIT + 1, vec![3]));
		<RateLimit<Runtime>>::put(100 * UNIT);
		<RateLimitState<Runtime>>::kill();

		// Unstake 100 UNIT + 1.
		assert_noop!(
			Staking::unstake(RuntimeOrigin::signed(1), 100 * UNIT + 1, Vec::new()),
			<Error<Runtime>>::ExceedRateLimit
		);

		// Unstake 100 UNIT + 1.
		assert_noop!(
			Staking::unstake(RuntimeOrigin::signed(1), 0, vec![3]),
			<Error<Runtime>>::ExceedRateLimit
		);

		// Unstake RING(100 UNIT + 1) and deposit(100 UNIT + 1).
		assert_noop!(
			Staking::unstake(RuntimeOrigin::signed(1), 100 * UNIT + 1, vec![3]),
			<Error<Runtime>>::ExceedRateLimit
		);
	});
}

#[test]
fn collect_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(Staking::collator_of(1).is_none());
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), UNIT, Vec::new()));

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
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), UNIT, Vec::new()));
		assert_ok!(Staking::collect(RuntimeOrigin::signed(1), Perbill::zero()));

		(2..=10).for_each(|n| {
			assert!(Staking::nominator_of(n).is_none());
			assert_ok!(Staking::stake(RuntimeOrigin::signed(n), UNIT, Vec::new()));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(n), 1));
			assert_eq!(Staking::nominator_of(n).unwrap(), 1);
		});
	});
}

#[test]
fn chill_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Staking::stake(RuntimeOrigin::signed(1), UNIT, Vec::new()));
		assert_ok!(Staking::collect(RuntimeOrigin::signed(1), Perbill::zero()));
		(2..=10).for_each(|n| {
			assert_ok!(Staking::stake(RuntimeOrigin::signed(n), UNIT, Vec::new()));
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
fn elect_should_work() {
	ExtBuilder::default().collator_count(3).build().execute_with(|| {
		(1..=5).for_each(|i| {
			assert_ok!(Staking::stake(RuntimeOrigin::signed(i), i as Balance * UNIT, Vec::new()));
			assert_ok!(Staking::collect(RuntimeOrigin::signed(i), Perbill::zero()));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(i), i));
		});
		(6..=10).for_each(|i| {
			assert_ok!(Staking::stake(RuntimeOrigin::signed(i), i as Balance * UNIT, Vec::new()));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(i), i - 5));
		});

		assert_eq!(Staking::elect().unwrap(), vec![5, 4, 3]);
	});
}

#[test]
fn payout_should_work() {
	ExtBuilder::default().collator_count(5).build().execute_with(|| {
		(1..=5).for_each(|i| {
			assert_ok!(Staking::stake(RuntimeOrigin::signed(i), i as Balance * UNIT, Vec::new()));
			assert_ok!(Staking::collect(RuntimeOrigin::signed(i), Perbill::from_percent(i * 10)));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(i), i));
		});
		(6..=10).for_each(|i| {
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(i),
				(11 - i as Balance) * UNIT,
				Vec::new()
			));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(i), i - 5));
		});
		new_session();
		(1..=10).for_each(|i| {
			assert_eq!(
				Balances::free_balance(i),
				(1_000 - if i < 6 { i } else { 11 - i }) as Balance * UNIT
			)
		});

		let session_duration = Duration::new(12 * 600, 0).as_millis();
		Efflux::time(session_duration - <Period as Get<u64>>::get() as Moment);
		Staking::note_authors(&[1, 2, 3, 4, 5]);
		new_session();
		new_session();
		payout();

		let rewards = [
			182149362040072859745,
			340012143096539162113,
			473588342440801457194,
			582877959635701275045,
			667880995628415300546,
			546448087213114754098,
			388585306229508196721,
			255009107468123861566,
			145719489836065573771,
			60716453916211293261,
		];
		assert_eq!(
			rewards.as_slice(),
			(1..=10)
				.map(|i| Balances::free_balance(i)
					- (1_000 - if i < 6 { i } else { 11 - i }) as Balance * UNIT)
				.collect::<Vec<_>>()
		);
		assert_eq_error_rate!(
			PayoutFraction::get()
				* dc_inflation::issuing_in_period(session_duration, Timestamp::now()).unwrap()
				/ 2,
			rewards.iter().sum::<Balance>(),
			// Error rate 1 RING.
			UNIT
		);
	});

	ExtBuilder::default().inflation_type(1).collator_count(5).build().execute_with(|| {
		(1..=5).for_each(|i| {
			assert_ok!(Staking::stake(RuntimeOrigin::signed(i), i as Balance * UNIT, Vec::new()));
			assert_ok!(Staking::collect(RuntimeOrigin::signed(i), Perbill::from_percent(i * 10)));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(i), i));
		});
		(6..=10).for_each(|i| {
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(i),
				(11 - i as Balance) * UNIT,
				Vec::new()
			));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(i), i - 5));
		});
		new_session();
		new_session();
		(1..=10).for_each(|i| {
			assert_eq!(
				Balances::free_balance(i),
				(1_000 - if i < 6 { i } else { 11 - i }) as Balance * UNIT
			)
		});

		let total_issuance = Balances::total_issuance();
		let session_duration = Duration::new(12 * 600, 0).as_millis();
		Efflux::time(session_duration - <Period as Get<u64>>::get() as Moment);
		Staking::note_authors(&[1, 2, 3, 4, 5]);
		new_session();
		payout();

		let rewards = [
			249999999400000000000,
			466666666400000000000,
			650000000000000000000,
			799999999600000000000,
			916666666500000000000,
			749999999700000000000,
			533333332800000000000,
			350000000000000000000,
			199999999800000000000,
			83333333000000000000,
		];
		assert_eq!(
			rewards.as_slice(),
			(1..=10)
				.map(|i| Balances::free_balance(i)
					- (1_000 - if i < 6 { i } else { 11 - i }) as Balance * UNIT)
				.collect::<Vec<_>>()
		);

		assert_eq!(Balances::total_issuance(), total_issuance);

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
				Event::Unpaid { who: 0, amount: 5000000000000000000000 },
				Event::Unpaid { who: 6, amount: 3749999998500000000000 },
				Event::Unpaid { who: 1, amount: 1249999997000000000000 }
			]
		);
	});
}

#[test]
fn auto_payout_should_work() {
	ExtBuilder::default().collator_count(2).build().execute_with(|| {
		(1..=2).for_each(|i| {
			assert_ok!(Staking::stake(RuntimeOrigin::signed(i), i as Balance * UNIT, Vec::new()));
			assert_ok!(Staking::collect(RuntimeOrigin::signed(i), Perbill::from_percent(i * 10)));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(i), i));
		});
		(3..=4).for_each(|i| {
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(i),
				(5 - i as Balance) * UNIT,
				Vec::new()
			));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(i), i - 2));
		});
		new_session();
		new_session();

		Efflux::time(<Period as Get<u64>>::get() as Moment);
		Staking::note_authors(&[1, 2]);
		new_session();
		(1..=4).for_each(|i| {
			assert_eq!(
				Balances::free_balance(i),
				(1_000 - if i < 3 { i } else { 5 - i }) as Balance * UNIT
			)
		});

		Efflux::block(1);
		assert_eq!(
			[
				Balances::free_balance(1),
				Balances::free_balance(2),
				Balances::free_balance(3),
				Balances::free_balance(4),
			],
			[
				999000607164541135398,
				998000000000000000000,
				998000910746811475409,
				999000000000000000000
			]
		);

		Efflux::block(1);
		assert_eq!(
			[
				Balances::free_balance(1),
				Balances::free_balance(2),
				Balances::free_balance(3),
				Balances::free_balance(4),
			],
			[
				999000607164541135398,
				998001113134992106860,
				998000910746811475409,
				999000404776360655738
			]
		);

		Efflux::block(1);
		assert_eq!(
			[
				Balances::free_balance(1),
				Balances::free_balance(2),
				Balances::free_balance(3),
				Balances::free_balance(4),
			],
			[
				999000607164541135398,
				998001113134992106860,
				998000910746811475409,
				999000404776360655738
			]
		);
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
		assert_ok!(Staking::stake(RuntimeOrigin::signed(3), 2 * UNIT, Vec::new()));
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
		assert_ok!(Staking::stake(RuntimeOrigin::signed(4), 2 * UNIT, Vec::new()));
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
		assert_ok!(Staking::stake(RuntimeOrigin::signed(5), 2 * UNIT, Vec::new()));
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
fn rate_limiter_should_work() {
	let r = RateLimiter::default();
	let r = r.flow_in(1, 3).unwrap();
	assert_eq!(r, RateLimiter::Pos(1));

	let r = r.flow_in(2, 3).unwrap();
	assert_eq!(r, RateLimiter::Pos(3));
	assert!(r.clone().flow_in(1, 3).is_none());

	let r = r.flow_out(1, 3).unwrap();
	assert_eq!(r, RateLimiter::Pos(2));

	let r = r.flow_out(2, 3).unwrap();
	assert_eq!(r, RateLimiter::Pos(0));

	let r = r.flow_out(1, 3).unwrap();
	assert_eq!(r, RateLimiter::Neg(1));

	let r = r.flow_out(2, 3).unwrap();
	assert_eq!(r, RateLimiter::Neg(3));
	assert!(r.clone().flow_out(1, 3).is_none());

	let r = r.flow_in(1, 3).unwrap();
	assert_eq!(r, RateLimiter::Neg(2));

	let r = r.flow_in(2, 3).unwrap();
	assert_eq!(r, RateLimiter::Neg(0));

	let r = r.flow_in(1, 3).unwrap();
	assert_eq!(r, RateLimiter::Pos(1));

	let r = RateLimiter::Pos(3);
	assert_eq!(r.flow_out(6, 3).unwrap(), RateLimiter::Neg(3));

	let r = RateLimiter::Neg(3);
	assert_eq!(r.flow_in(6, 3).unwrap(), RateLimiter::Pos(3));
}
