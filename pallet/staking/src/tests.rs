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
// polkadot-sdk
use frame_support::{assert_noop, assert_ok, BoundedVec};
use sp_runtime::{assert_eq_error_rate, DispatchError, Perbill};

#[test]
fn exposure_cache_should_work() {
	ExtBuilder::default().build().execute_with(|| {});
}

#[test]
fn stake_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(System::account(AccountId(1)).consumers, 0);
		assert!(Staking::ledger_of(AccountId(1)).is_none());
		assert_eq!(Balances::free_balance(AccountId(1)), 1_000 * UNIT);
		assert_eq!(Staking::rate_limit_state(), RateLimiter::Pos(0));

		// Stake 1 RING.
		assert_ok!(Staking::stake(RuntimeOrigin::signed(AccountId(1)), UNIT, Vec::new()));
		assert_eq!(System::account(AccountId(1)).consumers, 1);
		assert_eq!(
			Staking::ledger_of(AccountId(1)).unwrap(),
			Ledger { ring: UNIT, deposits: Default::default() }
		);
		assert_eq!(Balances::free_balance(AccountId(1)), 999 * UNIT);
		assert_eq!(Staking::rate_limit_state(), RateLimiter::Pos(UNIT));

		// Stake invalid deposit.
		assert_noop!(
			Staking::stake(RuntimeOrigin::signed(AccountId(1)), 0, vec![0]),
			<DepositError<Runtime>>::DepositNotFound
		);

		// Stake 1 deposit.
		assert_eq!(System::account(AccountId(1)).consumers, 1);
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(AccountId(1)), UNIT, 1));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(AccountId(1)), 0, vec![0]));
		assert_eq!(
			Staking::ledger_of(AccountId(1)).unwrap(),
			Ledger { ring: UNIT, deposits: BoundedVec::truncate_from(vec![0]) }
		);
		assert_eq!(Staking::rate_limit_state(), RateLimiter::Pos(2 * UNIT));

		// Stake 2 RING and 2 deposits.
		assert_eq!(System::account(AccountId(1)).consumers, 2);
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(AccountId(1)), UNIT, 1));
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(AccountId(1)), UNIT, 1));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(AccountId(1)), 2 * UNIT, vec![1, 2]));
		assert_eq!(Balances::free_balance(AccountId(1)), 994 * UNIT);
		assert_eq!(
			Staking::ledger_of(AccountId(1)).unwrap(),
			Ledger { ring: 3 * UNIT, deposits: BoundedVec::truncate_from(vec![0, 1, 2]) }
		);
		assert_eq!(Staking::rate_limit_state(), RateLimiter::Pos(6 * UNIT));
	});
}

#[test]
fn unstake_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(AccountId(1)), UNIT, 1));
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(AccountId(1)), UNIT, 1));
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(AccountId(1)), UNIT, 1));
		assert_ok!(Staking::stake(RuntimeOrigin::signed(AccountId(1)), 3 * UNIT, vec![0, 1, 2]));
		assert_eq!(Balances::free_balance(AccountId(1)), 994 * UNIT);
		assert_eq!(Staking::rate_limit_state(), RateLimiter::Pos(6 * UNIT));
		assert_eq!(
			Staking::ledger_of(AccountId(1)).unwrap(),
			Ledger { ring: 3 * UNIT, deposits: BoundedVec::truncate_from(vec![0, 1, 2]) }
		);
		assert_eq!(
			Deposit::deposit_of(AccountId(1))
				.unwrap()
				.into_iter()
				.map(|d| d.in_use)
				.collect::<Vec<_>>(),
			[true, true, true]
		);

		// Unstake 1 RING.
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(AccountId(1)), UNIT, Vec::new()));
		assert_eq!(Staking::rate_limit_state(), RateLimiter::Pos(5 * UNIT));
		assert_eq!(Balances::free_balance(AccountId(1)), 995 * UNIT);
		assert_eq!(
			Staking::ledger_of(AccountId(1)).unwrap(),
			Ledger { ring: 2 * UNIT, deposits: BoundedVec::truncate_from(vec![0, 1, 2]) }
		);

		// Unstake invalid deposit.
		assert_noop!(
			Staking::unstake(RuntimeOrigin::signed(AccountId(1)), 0, vec![3]),
			<Error<Runtime>>::DepositNotFound
		);

		// Unstake 1 deposit.
		Efflux::block(1);
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(AccountId(1)), 0, vec![1]));
		assert_eq!(Staking::rate_limit_state(), RateLimiter::Pos(4 * UNIT));
		assert_eq!(
			Staking::ledger_of(AccountId(1)).unwrap(),
			Ledger { ring: 2 * UNIT, deposits: BoundedVec::truncate_from(vec![0, 2]) }
		);
		assert_eq!(
			Deposit::deposit_of(AccountId(1))
				.unwrap()
				.into_iter()
				.map(|d| d.in_use)
				.collect::<Vec<_>>(),
			[true, false, true]
		);

		// Unstake 2 RING and 2 deposits.
		Efflux::block(1);
		assert_ok!(Staking::unstake(RuntimeOrigin::signed(AccountId(1)), 2 * UNIT, vec![0, 2]));
		assert_eq!(Staking::rate_limit_state(), RateLimiter::Pos(0));
		assert!(Staking::ledger_of(AccountId(1)).is_none());
		assert_eq!(
			Deposit::deposit_of(AccountId(1))
				.unwrap()
				.into_iter()
				.map(|d| d.in_use)
				.collect::<Vec<_>>(),
			[false, false, false]
		);

		// Prepare rate limit test data.
		assert_ok!(Deposit::lock(RuntimeOrigin::signed(AccountId(1)), 100 * UNIT + 1, 1));
		<RateLimit<Runtime>>::put(200 * UNIT + 2);
		assert_ok!(Staking::stake(RuntimeOrigin::signed(AccountId(1)), 100 * UNIT + 1, vec![3]));
		<RateLimit<Runtime>>::put(100 * UNIT);
		<RateLimitState<Runtime>>::kill();

		// Unstake 100 UNIT + 1.
		assert_noop!(
			Staking::unstake(RuntimeOrigin::signed(AccountId(1)), 100 * UNIT + 1, Vec::new()),
			<Error<Runtime>>::ExceedRateLimit
		);

		// Unstake 100 UNIT + 1.
		assert_noop!(
			Staking::unstake(RuntimeOrigin::signed(AccountId(1)), 0, vec![3]),
			<Error<Runtime>>::ExceedRateLimit
		);

		// Unstake RING(100 UNIT + 1) and deposit(100 UNIT + 1).
		assert_noop!(
			Staking::unstake(RuntimeOrigin::signed(AccountId(1)), 100 * UNIT + 1, vec![3]),
			<Error<Runtime>>::ExceedRateLimit
		);
	});
}

#[test]
fn collect_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(Staking::collator_of(AccountId(1)).is_none());
		assert_ok!(Staking::stake(RuntimeOrigin::signed(AccountId(1)), UNIT, Vec::new()));

		(0..=99).for_each(|c| {
			let c = Perbill::from_percent(c);

			assert_ok!(Staking::collect(RuntimeOrigin::signed(AccountId(1)), c));
			assert_eq!(Staking::collator_of(AccountId(1)).unwrap(), c);
		});
	});
}

#[test]
fn nominate_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Staking::stake(RuntimeOrigin::signed(AccountId(1)), UNIT, Vec::new()));
		assert_ok!(Staking::collect(RuntimeOrigin::signed(AccountId(1)), Perbill::zero()));

		(2..=10).for_each(|i| {
			assert!(Staking::nominator_of(AccountId(i)).is_none());
			assert_ok!(Staking::stake(RuntimeOrigin::signed(AccountId(i)), UNIT, Vec::new()));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(AccountId(i)), AccountId(1)));
			assert_eq!(Staking::nominator_of(AccountId(i)).unwrap(), AccountId(1));
		});
	});
}

#[test]
fn chill_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Staking::stake(RuntimeOrigin::signed(AccountId(1)), UNIT, Vec::new()));
		assert_ok!(Staking::collect(RuntimeOrigin::signed(AccountId(1)), Perbill::zero()));
		(2..=10).for_each(|i| {
			assert_ok!(Staking::stake(RuntimeOrigin::signed(AccountId(i)), UNIT, Vec::new()));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(AccountId(i)), AccountId(1)));
		});
		assert!(Staking::collator_of(AccountId(1)).is_some());
		(2..=10).for_each(|i| assert!(Staking::nominator_of(AccountId(i)).is_some()));

		(1..=10).for_each(|i| {
			assert_ok!(Staking::chill(RuntimeOrigin::signed(AccountId(i))));
		});
		assert!(Staking::collator_of(AccountId(1)).is_none());
		(2..=10).for_each(|i| assert!(Staking::nominator_of(AccountId(i)).is_none()));
	});
}

#[test]
fn set_collator_count_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Staking::set_collator_count(RuntimeOrigin::signed(AccountId(1)), 1),
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
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(AccountId(i)),
				i as Balance * UNIT,
				Vec::new()
			));
			assert_ok!(Staking::collect(RuntimeOrigin::signed(AccountId(i)), Perbill::zero()));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(AccountId(i)), AccountId(i)));
		});
		(6..=10).for_each(|i| {
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(AccountId(i)),
				i as Balance * UNIT,
				Vec::new()
			));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(AccountId(i)), AccountId(i - 5)));
		});

		assert_eq!(
			Staking::elect(Staking::collator_count()).unwrap(),
			vec![AccountId(5), AccountId(4), AccountId(3)]
		);
	});
}

#[test]
fn payout_should_work() {
	ExtBuilder::default().collator_count(5).build().execute_with(|| {
		(1..=5).for_each(|i| {
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(AccountId(i)),
				i as Balance * UNIT,
				Vec::new()
			));
			assert_ok!(Staking::collect(
				RuntimeOrigin::signed(AccountId(i)),
				Perbill::from_percent(i as u32 * 10)
			));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(AccountId(i)), AccountId(i)));
		});
		(6..=10).for_each(|i| {
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(AccountId(i)),
				(11 - i as Balance) * UNIT,
				Vec::new()
			));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(AccountId(i)), AccountId(i - 5)));
		});
		new_session();
		(1..=10).for_each(|i| {
			assert_eq!(
				Balances::free_balance(AccountId(i)),
				(1_000 - if i < 6 { i } else { 11 - i }) as Balance * UNIT
			)
		});

		let session_duration = Duration::new(12 * 600, 0).as_millis();
		let kton_staking_contract_balance =
			Balances::free_balance(<KtonStakingContract<Runtime>>::get().unwrap());
		Efflux::time(session_duration - <Period as Get<u64>>::get() as Moment);
		Staking::note_authors(&[
			AccountId(1),
			AccountId(2),
			AccountId(3),
			AccountId(4),
			AccountId(5),
		]);
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
		let half_reward = PAYOUT_FRAC
			* dc_inflation::issuing_in_period(session_duration, Timestamp::now()).unwrap()
			/ 2;
		assert_eq_error_rate!(half_reward, rewards.iter().sum::<Balance>(), UNIT);
		assert_eq_error_rate!(
			half_reward,
			Balances::free_balance(<KtonStakingContract<Runtime>>::get().unwrap())
				- kton_staking_contract_balance,
			UNIT
		);
		assert_eq!(
			rewards.as_slice(),
			(1..=10)
				.map(|i| Balances::free_balance(AccountId(i))
					- (1_000 - if i < 6 { i } else { 11 - i }) as Balance * UNIT)
				.collect::<Vec<_>>()
		);
	});

	ExtBuilder::default().inflation_type(1).collator_count(5).build().execute_with(|| {
		(1..=5).for_each(|i| {
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(AccountId(i)),
				i as Balance * UNIT,
				Vec::new()
			));
			assert_ok!(Staking::collect(
				RuntimeOrigin::signed(AccountId(i)),
				Perbill::from_percent(i as u32 * 10)
			));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(AccountId(i)), AccountId(i)));
		});
		(6..=10).for_each(|i| {
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(AccountId(i)),
				(11 - i as Balance) * UNIT,
				Vec::new()
			));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(AccountId(i)), AccountId(i - 5)));
		});
		new_session();
		new_session();
		(1..=10).for_each(|i| {
			assert_eq!(
				Balances::free_balance(AccountId(i)),
				(1_000 - if i < 6 { i } else { 11 - i }) as Balance * UNIT
			)
		});

		let total_issuance = Balances::total_issuance();
		let session_duration = Duration::new(12 * 600, 0).as_millis();
		Efflux::time(session_duration - <Period as Get<u64>>::get() as Moment);
		Staking::note_authors(&[
			AccountId(1),
			AccountId(2),
			AccountId(3),
			AccountId(4),
			AccountId(5),
		]);
		new_session();
		payout();

		let rewards = [
			499999998800000000000,
			933333332800000000000,
			1300000000000000000000,
			1599999999200000000000,
			1833333333000000000000,
			1499999999400000000000,
			1066666665600000000000,
			700000000000000000000,
			399999999600000000000,
			166666666000000000000,
		];
		assert_eq!(
			rewards.as_slice(),
			(1..=10)
				.map(|i| Balances::free_balance(AccountId(i))
					- (1_000 - if i < 6 { i } else { 11 - i }) as Balance * UNIT)
				.collect::<Vec<_>>()
		);

		assert_eq!(Balances::total_issuance(), total_issuance);

		assert_ok!(Balances::transfer_all(
			RuntimeOrigin::signed(Treasury::account_id()),
			AccountId(0),
			false
		));
		Staking::note_authors(&[AccountId(1)]);
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
				// Pay to collator failed.
				Event::Unpaid { who: AccountId(6), amount: 7499999997000000000000 },
				// Pay to nominator failed.
				Event::Unpaid { who: AccountId(1), amount: 2499999994000000000000 }
			]
		);
	});
}

#[test]
fn auto_payout_should_work() {
	ExtBuilder::default().collator_count(2).build().execute_with(|| {
		(1..=2).for_each(|i| {
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(AccountId(i)),
				i as Balance * UNIT,
				Vec::new()
			));
			assert_ok!(Staking::collect(
				RuntimeOrigin::signed(AccountId(i)),
				Perbill::from_percent(i as u32 * 10)
			));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(AccountId(i)), AccountId(i)));
		});
		(3..=4).for_each(|i| {
			assert_ok!(Staking::stake(
				RuntimeOrigin::signed(AccountId(i)),
				(5 - i as Balance) * UNIT,
				Vec::new()
			));
			assert_ok!(Staking::nominate(RuntimeOrigin::signed(AccountId(i)), AccountId(i - 2)));
		});
		new_session();
		new_session();

		Efflux::time(<Period as Get<u64>>::get() as Moment);
		Staking::note_authors(&[AccountId(1), AccountId(2)]);
		new_session();
		(1..=4).for_each(|i| {
			assert_eq!(
				Balances::free_balance(AccountId(i)),
				(1_000 - if i < 3 { i } else { 5 - i }) as Balance * UNIT
			)
		});

		Efflux::block(1);
		assert_eq!(
			[
				Balances::free_balance(AccountId(1)),
				Balances::free_balance(AccountId(2)),
				Balances::free_balance(AccountId(3)),
				Balances::free_balance(AccountId(4)),
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
				Balances::free_balance(AccountId(1)),
				Balances::free_balance(AccountId(2)),
				Balances::free_balance(AccountId(3)),
				Balances::free_balance(AccountId(4)),
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
				Balances::free_balance(AccountId(1)),
				Balances::free_balance(AccountId(2)),
				Balances::free_balance(AccountId(3)),
				Balances::free_balance(AccountId(4)),
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
	ExtBuilder::default().collator_count(2).genesis_collator().build().execute_with(|| {});
}

#[test]
fn get_top_collators_should_work() {
	const ZERO: [u8; 20] = [0; 20];

	let data = [
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 148, 244, 240, 74, 89, 79, 214, 144, 224,
		254, 164, 111, 40, 130, 165, 178, 97, 83, 167, 47, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	];
	#[allow(deprecated)]
	let function = Function {
		name: "getTopCollators".to_owned(),
		inputs: vec![Param {
			name: "k".to_owned(),
			kind: ParamType::Uint(256),
			internal_type: None,
		}],
		outputs: vec![Param {
			name: "collators".to_owned(),
			kind: ParamType::Array(Box::new(ParamType::Address)),
			internal_type: None,
		}],
		constant: None,
		state_mutability: StateMutability::View,
	};
	let output = function
		.decode_output(&data)
		.map(|o| {
			let Some(Token::Array(addrs)) = o.into_iter().next() else { return Vec::new() };

			addrs
				.into_iter()
				.filter_map(|addr| match addr {
					Token::Address(addr) if addr.0 != ZERO => Some(addr.0),
					_ => None,
				})
				.collect()
		})
		.unwrap();

	assert_eq!(
		output,
		[
			// 0x94f4f04a594fd690e0fea46f2882a5b26153a72f.
			[
				148, 244, 240, 74, 89, 79, 214, 144, 224, 254, 164, 111, 40, 130, 165, 178, 97, 83,
				167, 47
			]
		]
	);
}
