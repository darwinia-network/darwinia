use srml_support::{assert_err, assert_ok, traits::Currency};

use super::*;
use crate::mock::*;
use darwinia_support::{BalanceLock, NormalLock, StakingLock, WithdrawLock, WithdrawReason, WithdrawReasons};

// gen_paired_account!(a(1), b(2), m(12));
// will create stash `a` and controller `b`
// `a` has 100 Ring and 100 Kton
// promise for `m` month with 50 Ring and 50 Kton
// `m` can be ignore, and it wont perform `bond` action
// gen_paired_account!(a(1), b(2));
macro_rules! gen_paired_account {
	($stash:ident($stash_id:expr), $controller:ident($controller_id:expr), $promise_month:ident($how_long:expr)) => {
		#[allow(non_snake_case, unused)]
		let $stash = $stash_id;
		let _ = Ring::deposit_creating(&$stash, 100 * COIN);
		Kton::deposit_creating(&$stash, 100 * COIN);
		#[allow(non_snake_case, unused)]
		let $controller = $controller_id;
		let _ = Ring::deposit_creating(&$controller, COIN);
		#[allow(non_snake_case, unused)]
		let $promise_month = $how_long;
		assert_ok!(Staking::bond(
			Origin::signed($stash),
			$controller,
			StakingBalance::Ring(50 * COIN),
			RewardDestination::Stash,
			$how_long,
			));
		assert_ok!(Staking::bond_extra(
			Origin::signed($stash),
			StakingBalance::Kton(50 * COIN),
			$how_long
			));
	};
	($stash:ident($stash_id:expr), $controller:ident($controller_id:expr), $how_long:expr) => {
		#[allow(non_snake_case, unused)]
		let $stash = $stash_id;
		let _ = Ring::deposit_creating(&$stash, 100 * COIN);
		Kton::deposit_creating(&$stash, 100 * COIN);
		#[allow(non_snake_case, unused)]
		let $controller = $controller_id;
		let _ = Ring::deposit_creating(&$controller, COIN);
		assert_ok!(Staking::bond(
			Origin::signed($stash),
			$controller,
			StakingBalance::Ring(50 * COIN),
			RewardDestination::Stash,
			$how_long,
			));
		assert_ok!(Staking::bond_extra(
			Origin::signed($stash),
			StakingBalance::Kton(50 * COIN),
			$how_long,
			));
	};
	($stash:ident($stash_id:expr), $controller:ident($controller_id:expr)) => {
		#[allow(non_snake_case, unused)]
		let $stash = $stash_id;
		let _ = Ring::deposit_creating(&$stash, 100 * COIN);
		Kton::deposit_creating(&$stash, 100 * COIN);
		#[allow(non_snake_case, unused)]
		let $controller = $controller_id;
		let _ = Ring::deposit_creating(&$controller, COIN);
	};
}

#[test]
fn test_env_build() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		check_exposure_all();

		let (stash, controller) = (11, 10);

		assert_eq!(Staking::bonded(&stash), Some(controller));
		assert_eq!(
			Staking::ledger(&controller).unwrap(),
			StakingLedger {
				stash,
				active_ring: 100 * COIN,
				active_deposit_ring: 100 * COIN,
				active_kton: 0,
				deposit_items: vec![TimeDepositItem {
					value: 100 * COIN,
					start_time: 0,
					expire_time: (12 * MONTH_IN_SECONDS) as _,
				}],
				ring_staking_lock: StakingLock {
					staking_amount: 100 * COIN,
					unbondings: vec![],
				},
				kton_staking_lock: Default::default(),
			}
		);

		assert_eq!(Kton::free_balance(&11), COIN / 100);
		assert_eq!(Kton::total_issuance(), 16 * COIN / 100);

		let origin_ledger = Staking::ledger(&controller).unwrap();
		let _ = Ring::deposit_creating(&stash, 100 * COIN);
		assert_ok!(Staking::bond_extra(
			Origin::signed(stash),
			StakingBalance::Ring(20 * COIN),
			13,
		));
		assert_eq!(
			Staking::ledger(&controller).unwrap(),
			StakingLedger {
				stash,
				active_ring: origin_ledger.active_ring + 20 * COIN,
				active_deposit_ring: origin_ledger.active_deposit_ring + 20 * COIN,
				active_kton: 0,
				deposit_items: vec![
					TimeDepositItem {
						value: 100 * COIN,
						start_time: 0,
						expire_time: (12 * MONTH_IN_SECONDS) as _,
					},
					TimeDepositItem {
						value: 20 * COIN,
						start_time: 0,
						expire_time: (13 * MONTH_IN_SECONDS) as _,
					},
				],
				ring_staking_lock: StakingLock {
					staking_amount: origin_ledger.active_ring + 20 * COIN,
					unbondings: vec![],
				},
				kton_staking_lock: Default::default(),
			}
		);
	});
}

#[test]
fn normal_kton_should_work() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		{
			let (stash, controller) = (1001, 1000);

			Kton::deposit_creating(&stash, 10 * COIN);
			assert_ok!(Staking::bond(
				Origin::signed(stash),
				controller,
				StakingBalance::Kton(10 * COIN),
				RewardDestination::Stash,
				0,
			));
			assert_eq!(
				Staking::ledger(&controller).unwrap(),
				StakingLedger {
					stash,
					active_ring: 0,
					active_deposit_ring: 0,
					active_kton: 10 * COIN,
					deposit_items: vec![],
					ring_staking_lock: Default::default(),
					kton_staking_lock: StakingLock {
						staking_amount: 10 * COIN,
						unbondings: vec![],
					},
				}
			);
			assert_eq!(
				Kton::locks(&stash),
				vec![BalanceLock {
					id: STAKING_ID,
					withdraw_lock: WithdrawLock::WithStaking(StakingLock {
						staking_amount: 10 * COIN,
						unbondings: vec![],
					}),
					reasons: WithdrawReasons::all(),
				}]
			);
		}

		{
			let (stash, controller) = (2001, 2000);

			// promise_month should not work for kton
			Kton::deposit_creating(&stash, 10 * COIN);
			assert_ok!(Staking::bond(
				Origin::signed(stash),
				controller,
				StakingBalance::Kton(10 * COIN),
				RewardDestination::Stash,
				12,
			));
			assert_eq!(
				Staking::ledger(&controller).unwrap(),
				StakingLedger {
					stash,
					active_ring: 0,
					active_deposit_ring: 0,
					active_kton: 10 * COIN,
					deposit_items: vec![],
					ring_staking_lock: Default::default(),
					kton_staking_lock: StakingLock {
						staking_amount: 10 * COIN,
						unbondings: vec![],
					},
				}
			);
		}
	});
}

#[test]
fn time_deposit_ring_unbond_and_withdraw_automatically_should_work() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		let (stash, controller) = (11, 10);

		{
			let locks = vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 100 * COIN,
					unbondings: vec![],
				}),
				reasons: WithdrawReasons::all(),
			}];
			let ledger = StakingLedger {
				stash,
				active_ring: 100 * COIN,
				active_deposit_ring: 100 * COIN,
				active_kton: 0,
				deposit_items: vec![TimeDepositItem {
					value: 100 * COIN,
					start_time: 0,
					expire_time: (12 * MONTH_IN_SECONDS) as _,
				}],
				ring_staking_lock: StakingLock {
					staking_amount: 100 * COIN,
					unbondings: vec![],
				},
				kton_staking_lock: Default::default(),
			};

			assert_ok!(Staking::unbond(
				Origin::signed(controller),
				StakingBalance::Ring(10 * COIN)
			));
			assert_eq!(Ring::locks(stash), locks);
			assert_eq!(Staking::ledger(&controller).unwrap(), ledger,);

			assert_ok!(Staking::unbond(
				Origin::signed(controller),
				StakingBalance::Ring(120 * COIN)
			));
			assert_eq!(Ring::locks(stash), locks);
			assert_eq!(Staking::ledger(&controller).unwrap(), ledger);
		}

		{
			let (unbond_start, unbond_value) = ((13 * MONTH_IN_SECONDS) as _, 10 * COIN);
			Timestamp::set_timestamp(unbond_start);

			assert_ok!(Staking::unbond(
				Origin::signed(controller),
				StakingBalance::Ring(unbond_value)
			));
			assert_eq!(
				Ring::locks(stash),
				vec![BalanceLock {
					id: STAKING_ID,
					withdraw_lock: WithdrawLock::WithStaking(StakingLock {
						staking_amount: 100 * COIN - unbond_value,
						unbondings: vec![NormalLock {
							amount: unbond_value,
							until: unbond_start + BondingDuration::get(),
						}],
					}),
					reasons: WithdrawReasons::all(),
				}]
			);
			assert_eq!(
				Staking::ledger(&controller).unwrap(),
				StakingLedger {
					stash,
					active_ring: 100 * COIN - unbond_value,
					active_deposit_ring: 0,
					active_kton: 0,
					deposit_items: vec![],
					ring_staking_lock: StakingLock {
						staking_amount: 100 * COIN - unbond_value,
						unbondings: vec![NormalLock {
							amount: unbond_value,
							until: unbond_start + BondingDuration::get(),
						}],
					},
					kton_staking_lock: Default::default(),
				}
			);

			Timestamp::set_timestamp(unbond_start + BondingDuration::get());
			assert_err!(
				Ring::ensure_can_withdraw(
					&stash,
					unbond_value,
					WithdrawReason::Transfer.into(),
					100 * COIN - unbond_value - 1,
				),
				"account liquidity restrictions prevent withdrawal"
			);
			assert_ok!(Ring::ensure_can_withdraw(
				&stash,
				unbond_value,
				WithdrawReason::Transfer.into(),
				100 * COIN - unbond_value,
			));
		}
	});
}

#[test]
fn normal_unbond_should_work() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		let (stash, controller) = (11, 10);
		let value = 200 * COIN;
		let promise_month = 12;
		let _ = Ring::deposit_creating(&stash, 1000 * COIN);

		{
			let kton_free_balance = Kton::free_balance(&stash);
			let mut ledger = Staking::ledger(&controller).unwrap();

			assert_ok!(Staking::bond_extra(
				Origin::signed(stash),
				StakingBalance::Ring(value),
				promise_month,
			));
			assert_eq!(
				Kton::free_balance(&stash),
				kton_free_balance + inflation::compute_kton_return::<Test>(value, promise_month)
			);
			ledger.active_ring += value;
			ledger.active_deposit_ring += value;
			ledger.deposit_items.push(TimeDepositItem {
				value,
				start_time: 0,
				expire_time: (promise_month * MONTH_IN_SECONDS) as _,
			});
			ledger.ring_staking_lock.staking_amount += value;
			assert_eq!(Staking::ledger(&controller).unwrap(), ledger);
		}

		{
			let kton_free_balance = Kton::free_balance(&stash);
			let mut ledger = Staking::ledger(&controller).unwrap();

			// we try to bond 1 kton, but stash only has 0.03 Kton
			// extra = 1.min(0.03)
			// bond += 0.03
			assert_ok!(Staking::bond_extra(
				Origin::signed(stash),
				StakingBalance::Kton(COIN),
				0
			));
			ledger.active_kton += kton_free_balance;
			ledger.kton_staking_lock.staking_amount += kton_free_balance;
			assert_eq!(Staking::ledger(&controller).unwrap(), ledger);

			assert_ok!(Staking::unbond(
				Origin::signed(controller),
				StakingBalance::Kton(kton_free_balance)
			));
			ledger.active_kton = 0;
			ledger.kton_staking_lock.staking_amount = 0;
			ledger.kton_staking_lock.unbondings.push(NormalLock {
				amount: kton_free_balance,
				until: BondingDuration::get(),
			});
			assert_eq!(Staking::ledger(&controller).unwrap(), ledger);
		}
	});
}

#[test]
fn punished_claim_should_work() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		let (stash, controller) = (1001, 1000);
		let promise_month = 36;
		let _ = Ring::deposit_creating(&stash, 100 * COIN);
		Kton::deposit_creating(&stash, COIN / 100000);
		let mut ledger = StakingLedger {
			stash,
			active_ring: 10 * COIN,
			active_deposit_ring: 10 * COIN,
			active_kton: 0,
			deposit_items: vec![TimeDepositItem {
				value: 10 * COIN,
				start_time: 0,
				expire_time: (promise_month * MONTH_IN_SECONDS) as _,
			}],
			ring_staking_lock: StakingLock {
				staking_amount: 10 * COIN,
				unbondings: vec![],
			},
			kton_staking_lock: Default::default(),
		};

		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::Ring(10 * COIN),
			RewardDestination::Stash,
			promise_month,
		));
		assert_eq!(Staking::ledger(&controller).unwrap(), ledger);
		// kton is 0, skip unbond_with_punish
		assert_ok!(Staking::claim_deposits_with_punish(
			Origin::signed(controller),
			(promise_month * MONTH_IN_SECONDS) as _,
		));
		assert_eq!(Staking::ledger(&controller).unwrap(), ledger);

		// set more kton balance to make it work
		Kton::deposit_creating(&stash, 10 * COIN);
		let kton_free_balance = Kton::free_balance(&stash);
		let kton_punishment = inflation::compute_kton_return::<Test>(10 * COIN, promise_month);
		assert_ok!(Staking::claim_deposits_with_punish(
			Origin::signed(controller),
			(promise_month * MONTH_IN_SECONDS) as _,
		));
		ledger.active_deposit_ring -= 10 * COIN;
		ledger.deposit_items.clear();
		assert_eq!(Staking::ledger(&controller).unwrap(), ledger);
		assert_eq!(Kton::free_balance(&stash), kton_free_balance - 3 * kton_punishment);
	});
}

#[test]
fn transform_to_deposited_ring_should_work() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		let (stash, controller) = (1001, 1000);
		let _ = Ring::deposit_creating(&stash, 100 * COIN);
		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::Ring(10 * COIN),
			RewardDestination::Stash,
			0,
		));
		let kton_free_balance = Kton::free_balance(&stash);
		let mut ledger = Staking::ledger(&controller).unwrap();

		assert_ok!(Staking::deposit_extra(Origin::signed(controller), 5 * COIN, 12));
		ledger.active_deposit_ring += 5 * COIN;
		ledger.deposit_items.push(TimeDepositItem {
			value: 5 * COIN,
			start_time: 0,
			expire_time: (12 * MONTH_IN_SECONDS) as u64,
		});
		assert_eq!(Staking::ledger(&controller).unwrap(), ledger);
		assert_eq!(Kton::free_balance(&stash), kton_free_balance + (5 * COIN / 10000));
	});
}

#[test]
fn expired_ring_should_capable_to_promise_again() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		let (stash, controller) = (1001, 1000);
		let _ = Ring::deposit_creating(&stash, 100 * COIN);
		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::Ring(10 * COIN),
			RewardDestination::Stash,
			12,
		));
		let mut ledger = Staking::ledger(&controller).unwrap();
		let ts = (13 * MONTH_IN_SECONDS) as u64;
		let promise_extra_value = 5 * COIN;

		Timestamp::set_timestamp(ts);
		assert_ok!(Staking::deposit_extra(
			Origin::signed(controller),
			promise_extra_value,
			13,
		));
		ledger.active_deposit_ring = promise_extra_value;
		// old deposit_item with 12 months promised removed
		ledger.deposit_items = vec![TimeDepositItem {
			value: promise_extra_value,
			start_time: ts,
			expire_time: 2 * ts,
		}];
		assert_eq!(Staking::ledger(&controller).unwrap(), ledger);
	});
}

#[test]
fn inflation_should_be_correct() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		let initial_issuance = 1_200_000_000 * COIN;
		let surplus_needed = initial_issuance - Ring::total_issuance();
		let _ = Ring::deposit_into_existing(&11, surplus_needed);

		assert_eq!(Ring::total_issuance(), initial_issuance);
		// TODO
		//		assert_eq!(Staking::current_era_total_reward(), 80000000 * COIN / 10);
		//		start_era(11);
		//		// ErasPerEpoch = 10
		//		assert_eq!(Staking::current_era_total_reward(), 88000000 * COIN / 10);
	});
}

#[test]
fn reward_and_slash_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		gen_paired_account!(stash_1(123), _c(456), 12);
		gen_paired_account!(stash_2(234), _c(567), 12);

		<Stakers<Test>>::insert(
			&stash_1,
			Exposure {
				total: 1,
				own: 1,
				others: vec![],
			},
		);
		assert_eq!(Ring::free_balance(&stash_1), 100 * COIN);
		let _ = Staking::reward_validator(&stash_1, 20 * COIN);
		assert_eq!(Ring::free_balance(&stash_1), 120 * COIN);

		// FIXME: slash strategy
		//		<Stakers<Test>>::insert(
		//			&stash_1,
		//			Exposure {
		//				total: 100 * COIN,
		//				own: 1,
		//				others: vec![IndividualExposure {
		//					who: stash_2,
		//					value: 100 * COIN - 1,
		//				}],
		//			},
		//		);
		//		let _ = Staking::slash_validator(&stash_1, 1, &Staking::stakers(&stash_1), &mut Vec::new());
		//		assert_eq!(Ring::free_balance(&stash_1), 120 * COIN - 1);
		//		assert_eq!(Ring::free_balance(&stash_2), 1);
	});
}

#[test]
fn set_controller_should_work() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		let (stash, old_controller, new_controller) = (11, 10, 12);
		let ledger = Staking::ledger(&old_controller).unwrap();

		assert_ok!(Staking::set_controller(Origin::signed(stash), new_controller));
		assert_eq!(Staking::ledger(&old_controller), None);
		assert_eq!(Staking::ledger(&new_controller).unwrap(), ledger);
	});
}

#[test]
fn slash_should_not_touch_unbondings() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		let (stash, controller) = (11, 10);
		let ledger = Staking::ledger(&controller).unwrap();

		// only deposit_ring, no normal_ring
		assert_eq!(
			(ledger.active_ring, ledger.active_deposit_ring),
			(100 * COIN, 100 * COIN),
		);

		assert_ok!(Staking::bond_extra(
			Origin::signed(stash),
			StakingBalance::Ring(100 * COIN),
			0,
		));
		Kton::deposit_creating(&stash, 10 * COIN);
		assert_ok!(Staking::bond_extra(
			Origin::signed(stash),
			StakingBalance::Kton(10 * COIN),
			0,
		));

		assert_ok!(Staking::unbond(
			Origin::signed(controller),
			StakingBalance::Ring(10 * COIN),
		));
		let ledger = Staking::ledger(&controller).unwrap();
		let unbondings = (
			ledger.ring_staking_lock.unbondings.clone(),
			ledger.kton_staking_lock.unbondings.clone(),
		);
		assert_eq!(
			(ledger.active_ring, ledger.active_deposit_ring),
			(190 * COIN, 100 * COIN),
		);

		<Stakers<Test>>::insert(
			&stash,
			Exposure {
				total: 1,
				own: 1,
				others: vec![],
			},
		);
		// FIXME: slash strategy
		let _ = Staking::slash_validator(
			&stash,
			ExtendedBalance::max_value(),
			&Staking::stakers(&stash),
			&mut vec![],
		);
		let ledger = Staking::ledger(&controller).unwrap();
		assert_eq!(
			(
				ledger.ring_staking_lock.unbondings.clone(),
				ledger.kton_staking_lock.unbondings.clone(),
			),
			unbondings,
		);
		assert_eq!((ledger.active_ring, ledger.active_deposit_ring), (0, 0));
	});
}

#[test]
fn bond_over_max_promise_month_should_fail() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		gen_paired_account!(stash(123), controller(456));
		assert_err!(
			Staking::bond(
				Origin::signed(stash),
				controller,
				StakingBalance::Ring(COIN),
				RewardDestination::Stash,
				37,
			),
			"months at most is 36.",
		);

		gen_paired_account!(stash(123), controller(456), promise_month(12));
		assert_err!(
			Staking::bond_extra(Origin::signed(stash), StakingBalance::Ring(COIN), 37),
			"months at most is 36.",
		);
	});
}

#[test]
fn check_stash_already_bonded_and_controller_already_paired() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		gen_paired_account!(unpaired_stash(123), unpaired_controller(456));
		assert_err!(
			Staking::bond(
				Origin::signed(11),
				unpaired_controller,
				StakingBalance::Ring(COIN),
				RewardDestination::Stash,
				0,
			),
			"stash already bonded",
		);
		assert_err!(
			Staking::bond(
				Origin::signed(unpaired_stash),
				10,
				StakingBalance::Ring(COIN),
				RewardDestination::Stash,
				0,
			),
			"controller already paired",
		);
	});
}

#[test]
fn pool_should_be_increased_and_decreased_correctly() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		let mut ring_pool = Staking::ring_pool();
		let mut kton_pool = Staking::kton_pool();

		// bond: 100COIN
		gen_paired_account!(stash_1(111), controller_1(222), 0);
		gen_paired_account!(stash_2(333), controller_2(444), promise_month(12));
		ring_pool += 100 * COIN;
		kton_pool += 100 * COIN;
		assert_eq!(Staking::ring_pool(), ring_pool);
		assert_eq!(Staking::kton_pool(), kton_pool);

		// unbond: 50Ring 50Kton
		assert_ok!(Staking::unbond(
			Origin::signed(controller_1),
			StakingBalance::Ring(50 * COIN)
		));
		assert_ok!(Staking::unbond(
			Origin::signed(controller_1),
			StakingBalance::Kton(25 * COIN)
		));
		// not yet expired: promise for 12 months
		assert_ok!(Staking::unbond(
			Origin::signed(controller_2),
			StakingBalance::Ring(50 * COIN)
		));
		assert_ok!(Staking::unbond(
			Origin::signed(controller_2),
			StakingBalance::Kton(25 * COIN)
		));
		ring_pool -= 50 * COIN;
		kton_pool -= 50 * COIN;
		assert_eq!(Staking::ring_pool(), ring_pool);
		assert_eq!(Staking::kton_pool(), kton_pool);

		// claim: 50Ring
		assert_ok!(Staking::claim_deposits_with_punish(
			Origin::signed(controller_2),
			(promise_month * MONTH_IN_SECONDS) as u64,
		));
		// unbond deposit items: 12.5Ring
		Timestamp::set_timestamp((promise_month * MONTH_IN_SECONDS) as u64);
		assert_ok!(Staking::unbond(
			Origin::signed(controller_2),
			StakingBalance::Ring(125 * COIN / 10),
		));
		ring_pool -= 125 * COIN / 10;
		assert_eq!(Staking::ring_pool(), ring_pool);

		// slash: 37.5Ring 50Kton
		<Stakers<Test>>::insert(
			&stash_1,
			Exposure {
				total: 1,
				own: 1,
				others: vec![],
			},
		);
		<Stakers<Test>>::insert(
			&stash_2,
			Exposure {
				total: 1,
				own: 1,
				others: vec![],
			},
		);
		// FIXME: slash strategy
		let _ = Staking::slash_validator(
			&stash_1,
			ExtendedBalance::max_value(),
			&Staking::stakers(&stash_1),
			&mut vec![],
		);
		// FIXME: slash strategy
		let _ = Staking::slash_validator(
			&stash_2,
			ExtendedBalance::max_value(),
			&Staking::stakers(&stash_2),
			&mut vec![],
		);
		ring_pool -= 375 * COIN / 10;
		kton_pool -= 50 * COIN;
		assert_eq!(Staking::ring_pool(), ring_pool);
		assert_eq!(Staking::kton_pool(), kton_pool);
	});
}

#[test]
fn unbond_over_max_unbondings_chunks_should_fail() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		gen_paired_account!(stash(123), controller(456));
		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::Ring(COIN),
			RewardDestination::Stash,
			0,
		));

		for ts in 0..MAX_UNLOCKING_CHUNKS {
			Timestamp::set_timestamp(ts as u64);
			assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Ring(1)));
		}

		assert_err!(
			Staking::unbond(Origin::signed(controller), StakingBalance::Ring(1)),
			"can not schedule more unlock chunks",
		);
	});
}

#[test]
fn promise_extra_should_not_remove_unexpired_items() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		gen_paired_account!(stash(123), controller(456), promise_month(12));
		let expired_items_len = 3;
		let expiry_date = (promise_month * MONTH_IN_SECONDS) as u64;

		assert_ok!(Staking::bond_extra(
			Origin::signed(stash),
			StakingBalance::Ring(5 * COIN),
			0,
		));
		for _ in 0..expired_items_len {
			assert_ok!(Staking::deposit_extra(Origin::signed(controller), COIN, promise_month));
		}

		Timestamp::set_timestamp(expiry_date - 1);
		assert_ok!(Staking::deposit_extra(
			Origin::signed(controller),
			2 * COIN,
			promise_month,
		));
		assert_eq!(
			Staking::ledger(&controller).unwrap().deposit_items.len(),
			2 + expired_items_len,
		);

		Timestamp::set_timestamp(expiry_date);
		assert_ok!(Staking::deposit_extra(
			Origin::signed(controller),
			2 * COIN,
			promise_month,
		));
		assert_eq!(Staking::ledger(&controller).unwrap().deposit_items.len(), 2);
	});
}

#[test]
fn unbond_zero() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		gen_paired_account!(stash(123), controller(456), promise_month(12));
		let ledger = Staking::ledger(&controller).unwrap();

		Timestamp::set_timestamp((promise_month * MONTH_IN_SECONDS) as u64);
		assert_ok!(Staking::unbond(Origin::signed(10), StakingBalance::Ring(0)));
		assert_ok!(Staking::unbond(Origin::signed(10), StakingBalance::Kton(0)));
		assert_eq!(Staking::ledger(&controller).unwrap(), ledger);
	});
}

// bond 10_000 Ring for 12 months, gain 1 Kton
// bond extra 10_000 Ring for 36 months, gain 3 Kton
// bond extra 1 Kton
// nominate
// unlock the 12 months deposit item with punish
// lost 3 Kton and 10_000 Ring's power for nominate
#[test]
fn yakio_q1() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		let (stash, controller) = (777, 888);
		let _ = Ring::deposit_creating(&stash, 20_000);

		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::Ring(10_000),
			RewardDestination::Stash,
			12,
		));
		assert_ok!(Staking::bond_extra(
			Origin::signed(stash),
			StakingBalance::Ring(10_000),
			36,
		));
		assert_eq!(Kton::free_balance(&stash), 4);

		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Kton(1), 36));
		assert_eq!(Staking::ledger(&controller).unwrap().active_kton, 1);

		assert_ok!(Staking::nominate(Origin::signed(controller), vec![controller]));

		assert_ok!(Staking::claim_deposits_with_punish(
			Origin::signed(controller),
			(12 * MONTH_IN_SECONDS) as u64,
		));
		assert_eq!(Kton::free_balance(&stash), 1);

		let ledger = Staking::ledger(&controller).unwrap();
		// not enough Kton to unbond
		assert_ok!(Staking::claim_deposits_with_punish(
			Origin::signed(controller),
			(36 * MONTH_IN_SECONDS) as u64,
		));
		assert_eq!(Staking::ledger(&controller).unwrap(), ledger);
	});
}

// how to balance the power and calculate the reward if some validators have been chilled
#[test]
fn yakio_q2() {
	fn run(with_new_era: bool) -> Balance {
		let mut balance = 0;
		ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
			gen_paired_account!(validator_1_stash(123), validator_1_controller(456), 0);
			gen_paired_account!(validator_2_stash(234), validator_2_controller(567), 0);
			gen_paired_account!(nominator_stash(345), nominator_controller(678), 0);

			assert_ok!(Staking::validate(
				Origin::signed(validator_1_controller),
				ValidatorPrefs {
					node_name: vec![0; 8],
					..Default::default()
				},
			));
			assert_ok!(Staking::validate(
				Origin::signed(validator_2_controller),
				ValidatorPrefs {
					node_name: vec![1; 8],
					..Default::default()
				},
			));
			assert_ok!(Staking::nominate(
				Origin::signed(nominator_controller),
				vec![validator_1_stash, validator_2_stash],
			));

			start_era(1);
			assert_ok!(Staking::chill(Origin::signed(validator_1_controller)));
			// assert_ok!(Staking::chill(Origin::signed(validator_2_controller)));
			if with_new_era {
				start_era(2);
			}
			let _ = Staking::reward_validator(&validator_1_stash, 1000 * COIN);
			let _ = Staking::reward_validator(&validator_2_stash, 1000 * COIN);

			balance = Ring::free_balance(&nominator_stash);
		});

		balance
	}

	let free_balance = run(false);
	let free_balance_with_new_era = run(true);

	assert_ne!(free_balance, 0);
	assert_ne!(free_balance_with_new_era, 0);
	assert!(free_balance > free_balance_with_new_era);
}

#[test]
fn xavier_q1() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		let stash = 123;
		let controller = 456;
		Kton::deposit_creating(&stash, 10);

		Timestamp::set_timestamp(0);
		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::Kton(5),
			RewardDestination::Stash,
			0,
		));
		assert_eq!(Timestamp::get(), 0);
		assert_eq!(Kton::free_balance(stash), 10);
		assert_eq!(
			Kton::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 5,
					unbondings: vec![],
				}),
				reasons: WithdrawReasons::all()
			}]
		);
		//		println!("Ok Init - Kton Balance: {:?}", Kton::free_balance(stash));
		//		println!("Ok Init - Kton Locks: {:#?}", Kton::locks(stash));
		//		println!();

		Timestamp::set_timestamp(1);
		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Kton(5), 0));
		assert_eq!(Timestamp::get(), 1);
		assert_eq!(Kton::free_balance(stash), 10);
		assert_eq!(
			Kton::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 10,
					unbondings: vec![],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
		//		println!("Ok Bond Extra - Kton Balance: {:?}", Kton::free_balance(stash));
		//		println!("Ok Bond Extra - Kton Locks: {:#?}", Kton::locks(stash));
		//		println!();

		let unbond_start = 2;
		Timestamp::set_timestamp(unbond_start);
		assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Kton(9)));
		assert_eq!(Timestamp::get(), 2);
		assert_eq!(Kton::free_balance(stash), 10);
		assert_eq!(
			Kton::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 1,
					unbondings: vec![NormalLock {
						amount: 9,
						until: BondingDuration::get() + unbond_start,
					}],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
		//		println!("Ok Unbond - Kton Balance: {:?}", Kton::free_balance(stash));
		//		println!("Ok Unbond - Kton Locks: {:#?}", Kton::locks(stash));
		//		println!();

		assert_err!(
			Kton::transfer(Origin::signed(stash), controller, 1),
			"account liquidity restrictions prevent withdrawal"
		);
		//		println!("Locking Transfer - Kton Balance: {:?}", Kton::free_balance(stash));
		//		println!("Locking Transfer - Kton Locks: {:#?}", Kton::locks(stash));
		//		println!();

		Timestamp::set_timestamp(BondingDuration::get() + unbond_start);
		assert_ok!(Kton::transfer(Origin::signed(stash), controller, 1));
		//		println!("Unlocking Transfer - Kton Balance: {:?}", Kton::free_balance(stash));
		//		println!("Unlocking Transfer - Kton Locks: {:#?}", Kton::locks(stash));
		//		println!(
		//			"Unlocking Transfer - Kton StakingLedger: {:#?}",
		//			Staking::ledger(&controller)
		//		);
		//		println!();
		assert_eq!(Timestamp::get(), BondingDuration::get() + unbond_start);
		assert_eq!(Kton::free_balance(stash), 9);
		assert_eq!(
			Kton::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 1,
					unbondings: vec![NormalLock {
						amount: 9,
						until: BondingDuration::get() + unbond_start,
					}],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);

		Kton::deposit_creating(&stash, 20);
		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Kton(19), 0));
		assert_eq!(Kton::free_balance(stash), 29);
		assert_eq!(
			Kton::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 20,
					unbondings: vec![NormalLock {
						amount: 9,
						until: BondingDuration::get() + unbond_start,
					}],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
		assert_eq!(
			Staking::ledger(&controller).unwrap(),
			StakingLedger {
				stash: 123,
				active_ring: 0,
				active_deposit_ring: 0,
				active_kton: 20,
				deposit_items: vec![],
				ring_staking_lock: Default::default(),
				kton_staking_lock: StakingLock {
					staking_amount: 20,
					unbondings: vec![NormalLock {
						amount: 9,
						until: BondingDuration::get() + unbond_start,
					}],
				},
			}
		);
		//		println!("Unlocking Transfer - Kton Balance: {:?}", Kton::free_balance(stash));
		//		println!("Unlocking Transfer - Kton Locks: {:#?}", Kton::locks(stash));
		//		println!(
		//			"Unlocking Transfer - Kton StakingLedger: {:#?}",
		//			Staking::ledger(&controller)
		//		);
		//		println!();
	});

	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		let stash = 123;
		let controller = 456;
		let _ = Ring::deposit_creating(&stash, 10);

		Timestamp::set_timestamp(0);
		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::Ring(5),
			RewardDestination::Stash,
			0,
		));
		assert_eq!(Timestamp::get(), 0);
		assert_eq!(Ring::free_balance(stash), 10);
		assert_eq!(
			Ring::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 5,
					unbondings: vec![],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
		//		println!("Ok Init - Ring Balance: {:?}", Ring::free_balance(stash));
		//		println!("Ok Init - Ring Locks: {:#?}", Ring::locks(stash));
		//		println!();

		Timestamp::set_timestamp(1);
		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Ring(5), 0));
		assert_eq!(Timestamp::get(), 1);
		assert_eq!(Ring::free_balance(stash), 10);
		assert_eq!(
			Ring::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 10,
					unbondings: vec![],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
		//		println!("Ok Bond Extra - Ring Balance: {:?}", Ring::free_balance(stash));
		//		println!("Ok Bond Extra - Ring Locks: {:#?}", Ring::locks(stash));
		//		println!();

		let unbond_start = 2;
		Timestamp::set_timestamp(unbond_start);
		assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Ring(9)));
		assert_eq!(Timestamp::get(), 2);
		assert_eq!(Ring::free_balance(stash), 10);
		assert_eq!(
			Ring::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 1,
					unbondings: vec![NormalLock {
						amount: 9,
						until: BondingDuration::get() + unbond_start,
					}],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
		//		println!("Ok Unbond - Ring Balance: {:?}", Ring::free_balance(stash));
		//		println!("Ok Unbond - Ring Locks: {:#?}", Ring::locks(stash));
		//		println!();

		assert_err!(
			Ring::transfer(Origin::signed(stash), controller, 1),
			"account liquidity restrictions prevent withdrawal",
		);
		//		println!("Locking Transfer - Ring Balance: {:?}", Ring::free_balance(stash));
		//		println!("Locking Transfer - Ring Locks: {:#?}", Ring::locks(stash));
		//		println!();

		Timestamp::set_timestamp(BondingDuration::get() + unbond_start);
		assert_ok!(Ring::transfer(Origin::signed(stash), controller, 1));
		//		println!("Unlocking Transfer - Ring Balance: {:?}", Ring::free_balance(stash));
		//		println!("Unlocking Transfer - Ring Locks: {:#?}", Ring::locks(stash));
		//		println!(
		//			"Unlocking Transfer - Ring StakingLedger: {:#?}",
		//			Staking::ledger(&controller)
		//		);
		//		println!();
		assert_eq!(Timestamp::get(), BondingDuration::get() + unbond_start);
		assert_eq!(Ring::free_balance(stash), 9);
		assert_eq!(
			Ring::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 1,
					unbondings: vec![NormalLock {
						amount: 9,
						until: BondingDuration::get() + unbond_start,
					}],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);

		let _ = Ring::deposit_creating(&stash, 20);
		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Ring(19), 0));
		assert_eq!(Ring::free_balance(stash), 29);
		assert_eq!(
			Ring::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 20,
					unbondings: vec![NormalLock {
						amount: 9,
						until: BondingDuration::get() + unbond_start,
					}],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
		assert_eq!(
			Staking::ledger(&controller).unwrap(),
			StakingLedger {
				stash: 123,
				active_ring: 20,
				active_deposit_ring: 0,
				active_kton: 0,
				deposit_items: vec![],
				ring_staking_lock: StakingLock {
					staking_amount: 20,
					unbondings: vec![NormalLock {
						amount: 9,
						until: BondingDuration::get() + unbond_start,
					}]
				},
				kton_staking_lock: Default::default(),
			}
		);
		//		println!("Unlocking Transfer - Ring Balance: {:?}", Ring::free_balance(stash));
		//		println!("Unlocking Transfer - Ring Locks: {:#?}", Ring::locks(stash));
		//		println!(
		//			"Unlocking Transfer - Ring StakingLedger: {:#?}",
		//			Staking::ledger(&controller)
		//		);
		//		println!();
	});
}

#[test]
fn xavier_q2() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		let stash = 123;
		let controller = 456;
		Kton::deposit_creating(&stash, 10);

		Timestamp::set_timestamp(1);
		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::Kton(5),
			RewardDestination::Stash,
			0,
		));
		assert_eq!(Kton::free_balance(stash), 10);
		assert_eq!(
			Kton::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 5,
					unbondings: vec![],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
		//		println!("Ok Init - Kton Balance: {:?}", Kton::free_balance(stash));
		//		println!("Ok Init - Kton Locks: {:#?}", Kton::locks(stash));
		//		println!();

		Timestamp::set_timestamp(1);
		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Kton(4), 0));
		assert_eq!(Timestamp::get(), 1);
		assert_eq!(Kton::free_balance(stash), 10);
		assert_eq!(
			Kton::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 9,
					unbondings: vec![],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
		//		println!("Ok Bond Extra - Kton Balance: {:?}", Kton::free_balance(stash));
		//		println!("Ok Bond Extra - Kton Locks: {:#?}", Kton::locks(stash));
		//		println!();

		let (unbond_start_1, unbond_value_1) = (2, 2);
		Timestamp::set_timestamp(unbond_start_1);
		assert_ok!(Staking::unbond(
			Origin::signed(controller),
			StakingBalance::Kton(unbond_value_1),
		));
		assert_eq!(Timestamp::get(), unbond_start_1);
		assert_eq!(Kton::free_balance(stash), 10);
		assert_eq!(
			Kton::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 7,
					unbondings: vec![NormalLock {
						amount: 2,
						until: BondingDuration::get() + unbond_start_1,
					}],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
		//		println!("Ok Unbond - Kton Balance: {:?}", Kton::free_balance(stash));
		//		println!("Ok Unbond - Kton Locks: {:#?}", Kton::locks(stash));
		//		println!();

		let (unbond_start_2, unbond_value_2) = (3, 6);
		Timestamp::set_timestamp(unbond_start_2);
		assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Kton(6)));
		assert_eq!(Timestamp::get(), unbond_start_2);
		assert_eq!(Kton::free_balance(stash), 10);
		assert_eq!(
			Kton::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 1,
					unbondings: vec![
						NormalLock {
							amount: 2,
							until: BondingDuration::get() + unbond_start_1,
						},
						NormalLock {
							amount: 6,
							until: BondingDuration::get() + unbond_start_2,
						}
					],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
		//		println!("Ok Unbond - Kton Balance: {:?}", Kton::free_balance(stash));
		//		println!("Ok Unbond - Kton Locks: {:#?}", Kton::locks(stash));
		//		println!();

		assert_err!(
			Kton::transfer(Origin::signed(stash), controller, unbond_value_1),
			"account liquidity restrictions prevent withdrawal",
		);
		//		println!("Locking Transfer - Kton Balance: {:?}", Kton::free_balance(stash));
		//		println!("Locking Transfer - Kton Locks: {:#?}", Kton::locks(stash));
		//		println!();

		assert_ok!(Kton::transfer(Origin::signed(stash), controller, unbond_value_1 - 1));
		assert_eq!(Kton::free_balance(stash), 9);
		//		println!("Normal Transfer - Kton Balance: {:?}", Kton::free_balance(stash));
		//		println!("Normal Transfer - Kton Locks: {:#?}", Kton::locks(stash));

		Timestamp::set_timestamp(BondingDuration::get() + unbond_start_1);
		assert_err!(
			Kton::transfer(Origin::signed(stash), controller, unbond_value_1 + 1),
			"account liquidity restrictions prevent withdrawal",
		);
		//		println!("Locking Transfer - Kton Balance: {:?}", Kton::free_balance(stash));
		//		println!("Locking Transfer - Kton Locks: {:#?}", Kton::locks(stash));
		//		println!();
		assert_ok!(Kton::transfer(Origin::signed(stash), controller, unbond_value_1));
		assert_eq!(Timestamp::get(), BondingDuration::get() + unbond_start_1);
		assert_eq!(Kton::free_balance(stash), 7);
		assert_eq!(
			Kton::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 1,
					unbondings: vec![
						NormalLock {
							amount: 2,
							until: BondingDuration::get() + unbond_start_1,
						},
						NormalLock {
							amount: 6,
							until: BondingDuration::get() + unbond_start_2,
						}
					],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
		//		println!("Unlocking Transfer - Kton Balance: {:?}", Kton::free_balance(stash));
		//		println!("Unlocking Transfer - Kton Locks: {:#?}", Kton::locks(stash));

		Timestamp::set_timestamp(BondingDuration::get() + unbond_start_2);
		assert_ok!(Kton::transfer(Origin::signed(stash), controller, unbond_value_2));
		assert_eq!(Timestamp::get(), BondingDuration::get() + unbond_start_2);
		assert_eq!(Kton::free_balance(stash), 1);
		assert_eq!(
			Kton::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 1,
					unbondings: vec![
						NormalLock {
							amount: 2,
							until: BondingDuration::get() + unbond_start_1,
						},
						NormalLock {
							amount: 6,
							until: BondingDuration::get() + unbond_start_2,
						}
					],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
		//		println!("Unlocking Transfer - Kton Balance: {:?}", Kton::free_balance(stash));
		//		println!("Unlocking Transfer - Kton Locks: {:#?}", Kton::locks(stash));

		Kton::deposit_creating(&stash, 1);
		//		println!("Staking Ledger: {:#?}", Staking::ledger(controller).unwrap());
		assert_eq!(Kton::free_balance(stash), 2);
		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Kton(1), 0));
		assert_eq!(
			Kton::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 2,
					unbondings: vec![
						NormalLock {
							amount: 2,
							until: BondingDuration::get() + unbond_start_1,
						},
						NormalLock {
							amount: 6,
							until: BondingDuration::get() + unbond_start_2,
						}
					],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
	});

	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		let stash = 123;
		let controller = 456;
		let _ = Ring::deposit_creating(&stash, 10);

		Timestamp::set_timestamp(1);
		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::Ring(5),
			RewardDestination::Stash,
			0,
		));
		assert_eq!(Ring::free_balance(stash), 10);
		assert_eq!(
			Ring::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 5,
					unbondings: vec![],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
		//		println!("Ok Init - Ring Balance: {:?}", Ring::free_balance(stash));
		//		println!("Ok Init - Ring Locks: {:#?}", Ring::locks(stash));
		//		println!();

		Timestamp::set_timestamp(1);
		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Ring(4), 0));
		assert_eq!(Timestamp::get(), 1);
		assert_eq!(Ring::free_balance(stash), 10);
		assert_eq!(
			Ring::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 9,
					unbondings: vec![],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
		//		println!("Ok Bond Extra - Ring Balance: {:?}", Ring::free_balance(stash));
		//		println!("Ok Bond Extra - Ring Locks: {:#?}", Ring::locks(stash));
		//		println!();

		let (unbond_start_1, unbond_value_1) = (2, 2);
		Timestamp::set_timestamp(unbond_start_1);
		assert_ok!(Staking::unbond(
			Origin::signed(controller),
			StakingBalance::Ring(unbond_value_1)
		));
		assert_eq!(Timestamp::get(), unbond_start_1);
		assert_eq!(Ring::free_balance(stash), 10);
		assert_eq!(
			Ring::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 7,
					unbondings: vec![NormalLock {
						amount: 2,
						until: BondingDuration::get() + unbond_start_1,
					},],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
		//		println!("Ok Unbond - Ring Balance: {:?}", Ring::free_balance(stash));
		//		println!("Ok Unbond - Ring Locks: {:#?}", Ring::locks(stash));
		//		println!();

		let (unbond_start_2, unbond_value_2) = (3, 6);
		Timestamp::set_timestamp(unbond_start_2);
		assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Ring(6)));
		assert_eq!(Timestamp::get(), unbond_start_2);
		assert_eq!(Ring::free_balance(stash), 10);
		assert_eq!(
			Ring::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 1,
					unbondings: vec![
						NormalLock {
							amount: 2,
							until: BondingDuration::get() + unbond_start_1,
						},
						NormalLock {
							amount: 6,
							until: BondingDuration::get() + unbond_start_2,
						}
					],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
		//		println!("Ok Unbond - Ring Balance: {:?}", Ring::free_balance(stash));
		//		println!("Ok Unbond - Ring Locks: {:#?}", Ring::locks(stash));
		//		println!();

		assert_err!(
			Ring::transfer(Origin::signed(stash), controller, unbond_value_1),
			"account liquidity restrictions prevent withdrawal",
		);
		//		println!("Locking Transfer - Ring Balance: {:?}", Ring::free_balance(stash));
		//		println!("Locking Transfer - Ring Locks: {:#?}", Ring::locks(stash));
		//		println!();

		assert_ok!(Ring::transfer(Origin::signed(stash), controller, unbond_value_1 - 1));
		assert_eq!(Ring::free_balance(stash), 9);
		//		println!("Normal Transfer - Ring Balance: {:?}", Ring::free_balance(stash));
		//		println!("Normal Transfer - Ring Locks: {:#?}", Ring::locks(stash));

		Timestamp::set_timestamp(BondingDuration::get() + unbond_start_1);
		assert_err!(
			Ring::transfer(Origin::signed(stash), controller, unbond_value_1 + 1),
			"account liquidity restrictions prevent withdrawal",
		);
		//		println!("Locking Transfer - Ring Balance: {:?}", Ring::free_balance(stash));
		//		println!("Locking Transfer - Ring Locks: {:#?}", Ring::locks(stash));
		//		println!();
		assert_ok!(Ring::transfer(Origin::signed(stash), controller, unbond_value_1));
		assert_eq!(Timestamp::get(), BondingDuration::get() + unbond_start_1);
		assert_eq!(Ring::free_balance(stash), 7);
		assert_eq!(
			Ring::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 1,
					unbondings: vec![
						NormalLock {
							amount: 2,
							until: BondingDuration::get() + unbond_start_1,
						},
						NormalLock {
							amount: 6,
							until: BondingDuration::get() + unbond_start_2,
						}
					],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
		//		println!("Unlocking Transfer - Ring Balance: {:?}", Ring::free_balance(stash));
		//		println!("Unlocking Transfer - Ring Locks: {:#?}", Ring::locks(stash));

		Timestamp::set_timestamp(BondingDuration::get() + unbond_start_2);
		assert_ok!(Ring::transfer(Origin::signed(stash), controller, unbond_value_2));
		assert_eq!(Timestamp::get(), BondingDuration::get() + unbond_start_2);
		assert_eq!(Ring::free_balance(stash), 1);
		assert_eq!(
			Ring::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 1,
					unbondings: vec![
						NormalLock {
							amount: 2,
							until: BondingDuration::get() + unbond_start_1,
						},
						NormalLock {
							amount: 6,
							until: BondingDuration::get() + unbond_start_2,
						}
					],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
		//		println!("Unlocking Transfer - Ring Balance: {:?}", Ring::free_balance(stash));
		//		println!("Unlocking Transfer - Ring Locks: {:#?}", Ring::locks(stash));

		let _ = Ring::deposit_creating(&stash, 1);
		//		println!("Staking Ledger: {:#?}", Staking::ledger(controller).unwrap());
		assert_eq!(Ring::free_balance(stash), 2);
		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Ring(1), 0));
		assert_eq!(
			Ring::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 2,
					unbondings: vec![
						NormalLock {
							amount: 2,
							until: BondingDuration::get() + unbond_start_1,
						},
						NormalLock {
							amount: 6,
							until: BondingDuration::get() + unbond_start_2,
						}
					],
				}),
				reasons: WithdrawReasons::all(),
			}]
		);
	});
}

#[test]
fn xavier_q3() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		let stash = 123;
		let controller = 456;
		Kton::deposit_creating(&stash, 10);

		Timestamp::set_timestamp(1);
		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::Kton(5),
			RewardDestination::Stash,
			0,
		));
		assert_eq!(Timestamp::get(), 1);
		assert_eq!(
			Staking::ledger(&controller).unwrap(),
			StakingLedger {
				stash: 123,
				active_ring: 0,
				active_deposit_ring: 0,
				active_kton: 5,
				deposit_items: vec![],
				ring_staking_lock: Default::default(),
				kton_staking_lock: StakingLock {
					staking_amount: 5,
					unbondings: vec![],
				},
			}
		);
		//		println!("Locks: {:#?}", Kton::locks(stash));
		//		println!("StakingLedger: {:#?}", Staking::ledger(&controller));
		//		println!();

		assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Kton(5)));
		assert_eq!(
			Staking::ledger(&controller).unwrap(),
			StakingLedger {
				stash: 123,
				active_ring: 0,
				active_deposit_ring: 0,
				active_kton: 0,
				deposit_items: vec![],
				ring_staking_lock: Default::default(),
				kton_staking_lock: StakingLock {
					staking_amount: 0,
					unbondings: vec![NormalLock { amount: 5, until: 61 }],
				},
			}
		);
		//		println!("Locks: {:#?}", Kton::locks(stash));
		//		println!("StakingLedger: {:#?}", Staking::ledger(&controller));
		//		println!();

		Timestamp::set_timestamp(61);
		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Kton(1), 0));
		assert_eq!(Timestamp::get(), 61);
		assert_eq!(
			Staking::ledger(&controller).unwrap(),
			StakingLedger {
				stash: 123,
				active_ring: 0,
				active_deposit_ring: 0,
				active_kton: 1,
				deposit_items: vec![],
				ring_staking_lock: Default::default(),
				kton_staking_lock: StakingLock {
					staking_amount: 1,
					unbondings: vec![NormalLock { amount: 5, until: 61 }],
				},
			}
		);
		//		println!("Locks: {:#?}", Kton::locks(stash));
		//		println!("StakingLedger: {:#?}", Staking::ledger(&controller));
		//		println!();
	});

	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		let stash = 123;
		let controller = 456;
		let _ = Ring::deposit_creating(&stash, 10);

		Timestamp::set_timestamp(1);
		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::Ring(5),
			RewardDestination::Stash,
			0,
		));
		assert_eq!(Timestamp::get(), 1);
		assert_eq!(
			Staking::ledger(&controller).unwrap(),
			StakingLedger {
				stash: 123,
				active_ring: 5,
				active_deposit_ring: 0,
				active_kton: 0,
				deposit_items: vec![],
				ring_staking_lock: StakingLock {
					staking_amount: 5,
					unbondings: vec![],
				},
				kton_staking_lock: Default::default(),
			}
		);
		//		println!("Locks: {:#?}", Ring::locks(stash));
		//		println!("StakingLedger: {:#?}", Staking::ledger(&controller));
		//		println!();

		assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Ring(5)));
		assert_eq!(
			Staking::ledger(&controller).unwrap(),
			StakingLedger {
				stash: 123,
				active_ring: 0,
				active_deposit_ring: 0,
				active_kton: 0,
				deposit_items: vec![],
				ring_staking_lock: StakingLock {
					staking_amount: 0,
					unbondings: vec![NormalLock { amount: 5, until: 61 }],
				},
				kton_staking_lock: Default::default(),
			}
		);
		//		println!("Locks: {:#?}", Ring::locks(stash));
		//		println!("StakingLedger: {:#?}", Staking::ledger(&controller));
		//		println!();

		Timestamp::set_timestamp(61);
		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Ring(1), 0));
		assert_eq!(Timestamp::get(), 61);
		assert_eq!(
			Staking::ledger(&controller).unwrap(),
			StakingLedger {
				stash: 123,
				active_ring: 1,
				active_deposit_ring: 0,
				active_kton: 0,
				deposit_items: vec![],
				ring_staking_lock: StakingLock {
					staking_amount: 1,
					unbondings: vec![NormalLock { amount: 5, until: 61 }],
				},
				kton_staking_lock: Default::default(),
			}
		);
		//		println!("Locks: {:#?}", Ring::locks(stash));
		//		println!("StakingLedger: {:#?}", Staking::ledger(&controller));
		//		println!();
	});
}
