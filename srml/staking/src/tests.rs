use srml_support::{
	assert_err, assert_ok,
	traits::{Currency, WithdrawReason, WithdrawReasons},
};

use super::*;
use crate::mock::*;
use darwinia_support::{BalanceLock, NormalLock, StakingLock, WithdrawLock};

// gen_paired_account!(a(1), b(2), m(12));
// will create stash `a` and controller `b`
// `a` has 100 Ring and 100 Kton
// promise for `m` month with 50 Ring and 50 Kton
// `m` can be ignore, and it wont perform `bond` action
// gen_paired_account!(a(1), b(2));
//macro_rules! gen_paired_account {
//	($stash:ident($stash_id:expr), $controller:ident($controller_id:expr), $promise_month:ident($how_long:expr)) => {
//		#[allow(non_snake_case, unused)]
//		let $stash = $stash_id;
//		let _ = Ring::deposit_creating(&$stash, 100 * MILLICENTS);
//		Kton::deposit_creating(&$stash, 100 * MILLICENTS);
//		#[allow(non_snake_case, unused)]
//		let $controller = $controller_id;
//		let _ = Ring::deposit_creating(&$controller, MILLICENTS);
//		#[allow(non_snake_case, unused)]
//		let $promise_month = $how_long;
//		assert_ok!(Staking::bond(
//			Origin::signed($stash),
//			$controller,
//			StakingBalance::Ring(50 * MILLICENTS),
//			RewardDestination::Stash,
//			$how_long
//			));
//		assert_ok!(Staking::bond_extra(
//			Origin::signed($stash),
//			StakingBalance::Kton(50 * MILLICENTS),
//			$how_long
//			));
//	};
//	($stash:ident($stash_id:expr), $controller:ident($controller_id:expr), $how_long:expr) => {
//		#[allow(non_snake_case, unused)]
//		let $stash = $stash_id;
//		let _ = Ring::deposit_creating(&$stash, 100 * MILLICENTS);
//		Kton::deposit_creating(&$stash, 100 * MILLICENTS);
//		#[allow(non_snake_case, unused)]
//		let $controller = $controller_id;
//		let _ = Ring::deposit_creating(&$controller, MILLICENTS);
//		assert_ok!(Staking::bond(
//			Origin::signed($stash),
//			$controller,
//			StakingBalance::Ring(50 * MILLICENTS),
//			RewardDestination::Stash,
//			$how_long
//			));
//		assert_ok!(Staking::bond_extra(
//			Origin::signed($stash),
//			StakingBalance::Kton(50 * MILLICENTS),
//			$how_long
//			));
//	};
//	($stash:ident($stash_id:expr), $controller:ident($controller_id:expr)) => {
//		#[allow(non_snake_case, unused)]
//		let $stash = $stash_id;
//		let _ = Ring::deposit_creating(&$stash, 100 * MILLICENTS);
//		Kton::deposit_creating(&$stash, 100 * MILLICENTS);
//		#[allow(non_snake_case, unused)]
//		let $controller = $controller_id;
//		let _ = Ring::deposit_creating(&$controller, MILLICENTS);
//	};
//}

#[test]
fn test_env_build() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		check_exposure_all();

		assert_eq!(Staking::bonded(&11), Some(10));
		assert_eq!(
			Staking::ledger(&10).unwrap(),
			StakingLedger {
				stash: 11,
				active_ring: 100 * MILLICENTS,
				active_deposit_ring: 100 * MILLICENTS,
				active_kton: 0,
				deposit_items: vec![TimeDepositItem {
					value: 100 * MILLICENTS,
					start_time: 0,
					expire_time: 12 * MONTH_IN_SECONDS as u64
				}],
				ring_staking_lock: StakingLock {
					staking_amount: 100 * MILLICENTS,
					unbondings: vec![]
				},
				kton_staking_lock: StakingLock {
					staking_amount: 0,
					unbondings: vec![]
				},
			}
		);

		assert_eq!(Kton::free_balance(&11), MILLICENTS / 100);
		assert_eq!(Kton::total_issuance(), 16 * MILLICENTS / 100);

		let origin_ledger = Staking::ledger(&10).unwrap();
		let _ = Ring::deposit_creating(&11, 100 * MILLICENTS);
		assert_ok!(Staking::bond_extra(
			Origin::signed(11),
			StakingBalance::Ring(20 * MILLICENTS),
			13
		));
		assert_eq!(
			Staking::ledger(&10).unwrap(),
			StakingLedger {
				stash: 11,
				active_ring: origin_ledger.active_ring + 20 * MILLICENTS,
				active_deposit_ring: origin_ledger.active_deposit_ring + 20 * MILLICENTS,
				active_kton: 0,
				deposit_items: vec![
					TimeDepositItem {
						value: 100 * MILLICENTS,
						start_time: 0,
						expire_time: 12 * MONTH_IN_SECONDS as u64
					},
					TimeDepositItem {
						value: 20 * MILLICENTS,
						start_time: 0,
						expire_time: 13 * MONTH_IN_SECONDS as u64
					}
				],
				ring_staking_lock: StakingLock {
					staking_amount: origin_ledger.active_ring + 20 * MILLICENTS,
					unbondings: vec![]
				},
				kton_staking_lock: StakingLock {
					staking_amount: 0,
					unbondings: vec![]
				},
			}
		);
	});
}

#[test]
fn normal_kton_should_work() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		Kton::deposit_creating(&1001, 10 * MILLICENTS);
		assert_ok!(Staking::bond(
			Origin::signed(1001),
			1000,
			StakingBalance::Kton(10 * MILLICENTS),
			RewardDestination::Stash,
			0
		));
		assert_eq!(
			Staking::ledger(&1000).unwrap(),
			StakingLedger {
				stash: 1001,
				active_ring: 0,
				active_deposit_ring: 0,
				active_kton: 10 * MILLICENTS,
				deposit_items: vec![],
				ring_staking_lock: StakingLock {
					staking_amount: 0,
					unbondings: vec![]
				},
				kton_staking_lock: StakingLock {
					staking_amount: 10 * MILLICENTS,
					unbondings: vec![]
				},
			}
		);
		assert_eq!(
			Kton::locks(&1001),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 10 * MILLICENTS,
					unbondings: vec![],
				}),
				reasons: WithdrawReasons::all()
			}]
		);

		// promise_month should not work for kton
		Kton::deposit_creating(&2001, 10 * MILLICENTS);
		assert_ok!(Staking::bond(
			Origin::signed(2001),
			2000,
			StakingBalance::Kton(10 * MILLICENTS),
			RewardDestination::Stash,
			12
		));
		assert_eq!(
			Staking::ledger(&2000).unwrap(),
			StakingLedger {
				stash: 2001,
				active_ring: 0,
				active_deposit_ring: 0,
				active_kton: 10 * MILLICENTS,
				deposit_items: vec![],
				ring_staking_lock: StakingLock {
					staking_amount: 0,
					unbondings: vec![]
				},
				kton_staking_lock: StakingLock {
					staking_amount: 10 * MILLICENTS,
					unbondings: vec![]
				},
			}
		);
	});
}

#[test]
fn time_deposit_ring_unbond_and_withdraw_automactically_should_work() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		{
			let locks = vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 100 * MILLICENTS,
					unbondings: vec![],
				}),
				reasons: WithdrawReasons::all(),
			}];
			let ledger = StakingLedger {
				stash: 11,
				active_ring: 100 * MILLICENTS,
				active_deposit_ring: 100 * MILLICENTS,
				active_kton: 0,
				deposit_items: vec![TimeDepositItem {
					value: 100 * MILLICENTS,
					start_time: 0,
					expire_time: 12 * MONTH_IN_SECONDS as u64,
				}],
				ring_staking_lock: StakingLock {
					staking_amount: 100 * MILLICENTS,
					unbondings: vec![],
				},
				kton_staking_lock: StakingLock {
					staking_amount: 0,
					unbondings: vec![],
				},
			};

			assert_ok!(Staking::unbond(
				Origin::signed(10),
				StakingBalance::Ring(10 * MILLICENTS)
			));
			assert_eq!(Ring::locks(11), locks);
			assert_eq!(Staking::ledger(&10).unwrap(), ledger,);

			assert_ok!(Staking::unbond(
				Origin::signed(10),
				StakingBalance::Ring(120 * MILLICENTS)
			));
			assert_eq!(Ring::locks(11), locks);
			assert_eq!(Staking::ledger(&10).unwrap(), ledger);
		}

		let mut ts = 13 * MONTH_IN_SECONDS as u64;
		Timestamp::set_timestamp(ts);

		assert_ok!(Staking::unbond(
			Origin::signed(10),
			StakingBalance::Ring(10 * MILLICENTS)
		));
		ts += BondingDuration::get();
		assert_eq!(
			Ring::locks(11),
			vec![BalanceLock {
				id: STAKING_ID,
				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
					staking_amount: 90 * MILLICENTS,
					unbondings: vec![NormalLock {
						amount: 10 * MILLICENTS,
						until: ts,
					}],
				}),
				reasons: WithdrawReasons::all()
			}]
		);
		assert_eq!(
			Staking::ledger(&10).unwrap(),
			StakingLedger {
				stash: 11,
				active_ring: 90 * MILLICENTS,
				active_deposit_ring: 0,
				active_kton: 0,
				deposit_items: vec![],
				ring_staking_lock: StakingLock {
					staking_amount: 90 * MILLICENTS,
					unbondings: vec![NormalLock {
						amount: 10 * MILLICENTS,
						until: ts,
					}],
				},
				kton_staking_lock: StakingLock {
					staking_amount: 0,
					unbondings: vec![],
				},
			}
		);

		//		assert_eq!(
		//			Staking::ledger(&10).unwrap(),
		//			StakingLedger {
		//				stash: 11,
		//				active_ring: 0,
		//				active_deposit_ring: 0,
		//				active_kton: 0,
		//				deposit_items: vec![],
		//				ring_staking_lock: StakingLock {
		//					staking_amount: 0,
		//					unbondings: vec![]
		//				},
		//				kton_staking_lock: StakingLock {
		//					staking_amount: 0,
		//					unbondings: vec![]
		//				},
		//			}
		//		);
		//		unbondings: vec![]

		//		let free_balance = Ring::free_balance(&11);
		//		assert_eq!(
		//			Ring::locks(&11),
		//			vec![BalanceLock {
		//				id: STAKING_ID,
		//				withdraw_lock: WithdrawLock::WithStaking(StakingLock {
		//					staking_amount: 0,
		//					unbondings: vec![],
		//				}),
		//				reasons: WithdrawReasons::all()
		//			}]
		//		);
		//		assert_ok!(Ring::ensure_can_withdraw(
		//			&11,
		//			free_balance,
		//			WithdrawReason::Transfer.into(),
		//			0
		//		));
	});
}

//#[test]
//fn normal_unbond_should_work() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let stash = 11;
//		let controller = 10;
//		let value = 200 * MILLICENTS;
//		let promise_month = 12;
//		// unbond normal ring
//		let _ = Ring::deposit_creating(&stash, 1000 * MILLICENTS);
//
//		{
//			let kton_free_balance = Kton::free_balance(&stash);
//			let mut ledger = Staking::ledger(&controller).unwrap();
//
//			assert_ok!(Staking::bond_extra(
//				Origin::signed(stash),
//				StakingBalance::Ring(value),
//				promise_month,
//			));
//			assert_eq!(
//				Kton::free_balance(&stash),
//				kton_free_balance + inflation::compute_kton_return::<Test>(value, promise_month)
//			);
//			ledger.total_deposit_ring += value;
//			ledger.active_ring += value;
//			ledger.active_deposit_ring += value;
//			ledger.deposit_items.push(TimeDepositItem {
//				value: value,
//				start_time: 0,
//				expire_time: promise_month as u64 * MONTH_IN_SECONDS as u64,
//			});
//			assert_eq!(&Staking::ledger(&controller).unwrap(), &ledger);
//		}
//
//		{
//			let kton_free_balance = Kton::free_balance(&stash);
//			let mut ledger = Staking::ledger(&controller).unwrap();
//
//			// we try to bond 1 kton, but stash only has 0.03 Kton
//			// extra = 1.min(0.03)
//			// bond += 0.03
//			assert_ok!(Staking::bond_extra(
//				Origin::signed(stash),
//				StakingBalance::Kton(MILLICENTS),
//				0
//			));
//			ledger.active_kton += kton_free_balance;
//			assert_eq!(&Staking::ledger(&controller).unwrap(), &ledger);
//
//			assert_ok!(Staking::unbond(
//				Origin::signed(controller),
//				StakingBalance::Kton(kton_free_balance)
//			));
//			ledger.active_kton = 0;
//			ledger.unbondings = vec![NormalLock {
//				value: StakingBalance::Kton(kton_free_balance),
//				era: 3,
//				is_time_deposit: false,
//			}];
//			assert_eq!(&Staking::ledger(&controller).unwrap(), &ledger);
//		}
//	});
//}
//
//#[test]
//fn punished_unbond_should_work() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let stash = 1001;
//		let controller = 1000;
//		let promise_month = 36;
//
//		let _ = Ring::deposit_creating(&stash, 100 * MILLICENTS);
//		Kton::deposit_creating(&stash, MILLICENTS / 100000);
//
//		// timestamp now is 0.
//		// free balance of kton is too low to work
//		assert_ok!(Staking::bond(
//			Origin::signed(stash),
//			controller,
//			StakingBalance::Ring(10 * MILLICENTS),
//			RewardDestination::Stash,
//			promise_month
//		));
//		assert_eq!(
//			Staking::ledger(&controller),
//			Some(StakingLedger {
//				stash,
//				total_deposit_ring: 10 * MILLICENTS,
//				active_deposit_ring: 10 * MILLICENTS,
//				active_ring: 10 * MILLICENTS,
//				active_kton: 0,
//				deposit_items: vec![TimeDepositItem {
//					value: 10 * MILLICENTS,
//					start_time: 0,
//					expire_time: promise_month as u64 * MONTH_IN_SECONDS as u64
//				}], // should be cleared
//				unbondings: vec![]
//			})
//		);
//		let mut ledger = Staking::ledger(&controller).unwrap();
//		let kton_free_balance = Kton::free_balance(&stash);
//		// kton is 0, skip unbond_with_punish
//		assert_ok!(Staking::unbond_with_punish(
//			Origin::signed(controller),
//			10 * MILLICENTS,
//			promise_month as u64 * MONTH_IN_SECONDS as u64
//		));
//		assert_eq!(&Staking::ledger(&controller).unwrap(), &ledger);
//		assert_eq!(Kton::free_balance(&stash), kton_free_balance);
//
//		// set more kton balance to make it work
//		Kton::deposit_creating(&stash, 10 * MILLICENTS);
//		let kton_free_balance = Kton::free_balance(&stash);
//		let unbond_value = 5 * MILLICENTS;
//		assert_ok!(Staking::unbond_with_punish(
//			Origin::signed(controller),
//			unbond_value,
//			promise_month as u64 * MONTH_IN_SECONDS as u64
//		));
//		ledger.active_ring -= unbond_value;
//		ledger.active_deposit_ring -= unbond_value;
//		ledger.deposit_items[0].value -= unbond_value;
//		ledger.unbondings = vec![NormalLock {
//			value: StakingBalance::Ring(unbond_value),
//			era: 3,
//			is_time_deposit: true,
//		}];
//		assert_eq!(&Staking::ledger(&controller).unwrap(), &ledger);
//
//		let kton_punishment = inflation::compute_kton_return::<Test>(unbond_value, promise_month);
//		assert_eq!(Kton::free_balance(&stash), kton_free_balance - 3 * kton_punishment);
//
//		// if deposit_item.value == 0
//		// the whole item should be be dropped
//		assert_ok!(Staking::unbond_with_punish(
//			Origin::signed(controller),
//			5 * MILLICENTS,
//			promise_month as u64 * MONTH_IN_SECONDS as u64
//		));
//		assert!(Staking::ledger(&controller).unwrap().deposit_items.is_empty());
//	});
//}
//
//#[test]
//fn transform_to_promised_ring_should_work() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let _ = Ring::deposit_creating(&1001, 100 * MILLICENTS);
//		assert_ok!(Staking::bond(
//			Origin::signed(1001),
//			1000,
//			StakingBalance::Ring(10 * MILLICENTS),
//			RewardDestination::Stash,
//			0
//		));
//		let origin_ledger = Staking::ledger(&1000).unwrap();
//		let kton_free_balance = Kton::free_balance(&1001);
//
//		assert_ok!(Staking::promise_extra(Origin::signed(1000), 5 * MILLICENTS, 12));
//
//		assert_eq!(
//			Staking::ledger(&1000),
//			Some(StakingLedger {
//				stash: 1001,
//				total_deposit_ring: origin_ledger.total_deposit_ring + 5 * MILLICENTS,
//				active_deposit_ring: origin_ledger.active_deposit_ring + 5 * MILLICENTS,
//				active_ring: origin_ledger.active_ring,
//				active_kton: origin_ledger.active_kton,
//				deposit_items: vec![TimeDepositItem {
//					value: 5 * MILLICENTS,
//					start_time: 0,
//					expire_time: 12 * MONTH_IN_SECONDS as u64
//				}],
//				unbondings: vec![]
//			})
//		);
//
//		assert_eq!(Kton::free_balance(&1001), kton_free_balance + (5 * MILLICENTS / 10000));
//	});
//}
//
//#[test]
//fn expired_ring_should_capable_to_promise_again() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let _ = Ring::deposit_creating(&1001, 100 * MILLICENTS);
//		assert_ok!(Staking::bond(
//			Origin::signed(1001),
//			1000,
//			StakingBalance::Ring(10 * MILLICENTS),
//			RewardDestination::Stash,
//			12
//		));
//		let mut ledger = Staking::ledger(&1000).unwrap();
//		let ts = 13 * MONTH_IN_SECONDS as u64;
//		let promise_extra_value = 5 * MILLICENTS;
//		Timestamp::set_timestamp(ts);
//		assert_ok!(Staking::promise_extra(Origin::signed(1000), promise_extra_value, 13));
//		ledger.total_deposit_ring = promise_extra_value;
//		ledger.active_deposit_ring = promise_extra_value;
//		// old deposit_item with 12 months promised removed
//		ledger.deposit_items = vec![TimeDepositItem {
//			value: promise_extra_value,
//			start_time: ts,
//			expire_time: 2 * ts,
//		}];
//		assert_eq!(&Staking::ledger(&1000).unwrap(), &ledger);
//	});
//}
//
////#[test]
////fn inflation_should_be_correct() {
////	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
////		let initial_issuance = 1_200_000_000 * MILLICENTS;
////		let surplus_needed = initial_issuance - Ring::total_issuance();
////		let _ = Ring::deposit_into_existing(&11, surplus_needed);
////		assert_eq!(Ring::total_issuance(), initial_issuance);
////		//		assert_eq!(Staking::current_era_total_reward(), 80000000 * MILLICENTS / 10);
////		start_era(11);
////		// ErasPerEpoch = 10
////		//		assert_eq!(Staking::current_era_total_reward(), 88000000 * MILLICENTS / 10);
////	});
////}
//
//#[test]
//fn reward_should_work_correctly() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		// create controller account
//		let _ = Ring::deposit_creating(&2000, MILLICENTS);
//		let _ = Ring::deposit_creating(&1000, MILLICENTS);
//		let _ = Ring::deposit_creating(&200, MILLICENTS);
//		// new validator
//		let _ = Ring::deposit_creating(&2001, 2000 * MILLICENTS);
//		Kton::deposit_creating(&2001, 10 * MILLICENTS);
//		// new validator
//		let _ = Ring::deposit_creating(&1001, 300 * MILLICENTS);
//		Kton::deposit_creating(&1001, 1 * MILLICENTS);
//		// handle some dirty work
//		let _ = Ring::deposit_creating(&201, 2000 * MILLICENTS);
//		Kton::deposit_creating(&201, 10 * MILLICENTS);
//		assert_eq!(Kton::free_balance(&201), 10 * MILLICENTS);
//
//		// 2001-2000
//		assert_ok!(Staking::bond(
//			Origin::signed(2001),
//			2000,
//			StakingBalance::Ring(300 * MILLICENTS),
//			RewardDestination::Controller,
//			12,
//		));
//		assert_ok!(Staking::bond_extra(
//			Origin::signed(2001),
//			StakingBalance::Kton(1 * MILLICENTS),
//			0
//		));
//		// 1001-1000
//		assert_ok!(Staking::bond(
//			Origin::signed(1001),
//			1000,
//			StakingBalance::Ring(300 * MILLICENTS),
//			RewardDestination::Controller,
//			12,
//		));
//		assert_ok!(Staking::bond_extra(
//			Origin::signed(1001),
//			StakingBalance::Kton(1 * MILLICENTS),
//			0
//		));
//		let ring_pool = Staking::ring_pool();
//		let kton_pool = Staking::kton_pool();
//		// 201-200
//		assert_ok!(Staking::bond(
//			Origin::signed(201),
//			200,
//			StakingBalance::Ring(3000 * MILLICENTS - ring_pool),
//			RewardDestination::Stash,
//			12,
//		));
//		assert_ok!(Staking::bond_extra(
//			Origin::signed(201),
//			StakingBalance::Kton(10 * MILLICENTS - kton_pool),
//			0,
//		));
//		// ring_pool and kton_pool
//		assert_eq!(Staking::ring_pool(), 3000 * MILLICENTS);
//		assert_eq!(Staking::kton_pool(), 10 * MILLICENTS);
//		// 1/5 ring_pool and 1/5 kton_pool
//		assert_ok!(Staking::validate(Origin::signed(2000), [0; 8].to_vec(), 0, 3));
//		assert_ok!(Staking::nominate(Origin::signed(1000), vec![2001]));
//
//		assert_eq!(Staking::ledger(&2000).unwrap().active_kton, 1 * MILLICENTS);
//		assert_eq!(Staking::ledger(&2000).unwrap().active_ring, 300 * MILLICENTS);
//		assert_eq!(Staking::power_of(&2001), 1_000_000_000 / 10 as u128);
//		// 600COIN for rewarding ring bond-er
//		// 600COIN for rewarding kton bond-er
//		Staking::select_validators();
//		Staking::reward_validator(&2001, 1200 * MILLICENTS);
//
//		assert_eq!(
//			Staking::stakers(2001),
//			Exposure {
//				total: 1200000000000,
//				own: 600000000000,
//				others: vec![IndividualExposure {
//					who: 1001,
//					value: 600000000000
//				}]
//			}
//		);
//		assert_eq!(Ring::free_balance(&2000), 601 * MILLICENTS);
//		assert_eq!(Ring::free_balance(&1000), 601 * MILLICENTS);
//	});
//}
//
//#[test]
//fn slash_should_work() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let _ = Ring::deposit_creating(&1001, 100 * MILLICENTS);
//		Kton::deposit_creating(&1001, 100 * MILLICENTS);
//
//		assert_ok!(Staking::bond(
//			Origin::signed(1001),
//			1000,
//			StakingBalance::Ring(50 * MILLICENTS),
//			RewardDestination::Controller,
//			0,
//		));
//		assert_ok!(Staking::bond_extra(
//			Origin::signed(1001),
//			StakingBalance::Kton(50 * MILLICENTS),
//			0
//		));
//		assert_ok!(Staking::validate(Origin::signed(1000), [0; 8].to_vec(), 0, 3));
//
//		// slash 1%
//		let slash_value = 5 * MILLICENTS / 10;
//		let mut ledger = Staking::ledger(&1000).unwrap();
//		let ring_free_balance = Ring::free_balance(&1001);
//		let kton_free_balance = Kton::free_balance(&1001);
//		Staking::slash_validator(&1001, 10_000_000);
//		ledger.active_ring -= slash_value;
//		ledger.active_kton -= slash_value;
//		assert_eq!(&Staking::ledger(&1000).unwrap(), &ledger);
//		assert_eq!(Ring::free_balance(&1001), ring_free_balance - slash_value);
//		assert_eq!(Kton::free_balance(&1001), kton_free_balance - slash_value);
//	});
//}
//
//#[test]
////fn test_inflation() {
////	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
////		assert_eq!(Staking::current_era_total_reward(), 80_000_000 * MILLICENTS / 10);
////		start_era(20);
////		assert_eq!(Staking::epoch_index(), 2);
////		assert_eq!(Staking::current_era_total_reward(), 9_999_988_266 * MILLICENTS / 1000);
////	});
////}
//#[test]
//fn set_controller_should_remove_old_ledger() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let stash = 11;
//		let old_controller = 10;
//		let new_controller = 12;
//
//		assert!(Staking::ledger(&old_controller).is_some());
//		assert_eq!(Staking::bonded(&stash), Some(old_controller));
//
//		assert_ok!(Staking::set_controller(Origin::signed(stash), new_controller));
//		assert!(Staking::ledger(&old_controller).is_none());
//	});
//}
//
//#[test]
//fn set_controller_should_not_change_ledger() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		assert_eq!(Staking::ledger(&10).unwrap().active_ring, 100 * MILLICENTS);
//		assert_ok!(Staking::set_controller(Origin::signed(11), 12));
//		assert_eq!(Staking::ledger(&12).unwrap().active_ring, 100 * MILLICENTS);
//	});
//}
//
//#[test]
//fn slash_should_not_touch_unbondingss() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let old_ledger = Staking::ledger(&10).unwrap();
//		// only deposit_ring, no normal_ring
//		assert_eq!(
//			(
//				old_ledger.active_ring,
//				old_ledger.active_deposit_ring
//			),
//			(100 * MILLICENTS, 100 * MILLICENTS)
//		);
//
//		assert_ok!(Staking::bond_extra(
//			Origin::signed(11),
//			StakingBalance::Ring(100 * MILLICENTS),
//			0
//		));
//		Kton::deposit_creating(&11, 10 * MILLICENTS);
//		assert_ok!(Staking::bond_extra(
//			Origin::signed(11),
//			StakingBalance::Kton(10 * MILLICENTS),
//			0
//		));
//
//		assert_ok!(Staking::unbond(Origin::signed(10), StakingBalance::Ring(10 * MILLICENTS)));
//		let new_ledger = Staking::ledger(&10).unwrap();
//		assert_eq!(
//			(
//				new_ledger.active_ring,
//				new_ledger.active_deposit_ring
//			),
//			(190 * MILLICENTS, 100 * MILLICENTS)
//		);
//
//		// slash 100%
//		Staking::slash_validator(&11, 1_000_000_000);
//
//		let ledger = Staking::ledger(&10).unwrap();
//		assert_eq!(
//			(ledger.active_ring, ledger.active_deposit_ring),
//			// 10Ring in unbondings
//			(0, 0)
//		);
//		assert_eq!(ledger.unbondings[0].value, StakingBalance::Ring(10 * MILLICENTS));
//	});
//}
//
//#[test]
//fn bond_over_max_promise_month_should_fail() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		gen_paired_account!(stash(123), controller(456));
//		assert_err!(
//			Staking::bond(
//				Origin::signed(stash),
//				controller,
//				StakingBalance::Ring(MILLICENTS),
//				RewardDestination::Stash,
//				37
//			),
//			"months at most is 36."
//		);
//
//		gen_paired_account!(stash(123), controller(456), promise_month(12));
//		assert_err!(
//			Staking::bond_extra(Origin::signed(stash), StakingBalance::Ring(MILLICENTS), 37),
//			"months at most is 36."
//		);
//	});
//}
//
//#[test]
//fn stash_already_bonded_and_controller_already_paired_should_fail() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		gen_paired_account!(unpaired_stash(123), unpaired_controller(456));
//		assert_err!(
//			Staking::bond(
//				Origin::signed(11),
//				unpaired_controller,
//				StakingBalance::Ring(MILLICENTS),
//				RewardDestination::Stash,
//				0
//			),
//			"stash already bonded"
//		);
//		assert_err!(
//			Staking::bond(
//				Origin::signed(unpaired_stash),
//				10,
//				StakingBalance::Ring(MILLICENTS),
//				RewardDestination::Stash,
//				0
//			),
//			"controller already paired"
//		);
//	});
//}
//
//#[test]
//fn pool_should_be_increased_and_decreased_correctly() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let mut ring_pool = Staking::ring_pool();
//		let mut kton_pool = Staking::kton_pool();
//
//		// bond: 100COIN
//		gen_paired_account!(stash_1(111), controller_1(222), 0);
//		gen_paired_account!(stash_2(333), controller_2(444), promise_month(12));
//		ring_pool += 100 * MILLICENTS;
//		kton_pool += 100 * MILLICENTS;
//		assert_eq!(Staking::ring_pool(), ring_pool);
//		assert_eq!(Staking::kton_pool(), kton_pool);
//
//		// unbond: 50Ring 50Kton
//		assert_ok!(Staking::unbond(
//			Origin::signed(controller_1),
//			StakingBalance::Ring(50 * MILLICENTS)
//		));
//		assert_ok!(Staking::unbond(
//			Origin::signed(controller_1),
//			StakingBalance::Kton(25 * MILLICENTS)
//		));
//		// not yet expired: promise for 12 months
//		assert_ok!(Staking::unbond(
//			Origin::signed(controller_2),
//			StakingBalance::Ring(50 * MILLICENTS)
//		));
//		assert_ok!(Staking::unbond(
//			Origin::signed(controller_2),
//			StakingBalance::Kton(25 * MILLICENTS)
//		));
//		ring_pool -= 50 * MILLICENTS;
//		kton_pool -= 50 * MILLICENTS;
//		assert_eq!(Staking::ring_pool(), ring_pool);
//		assert_eq!(Staking::kton_pool(), kton_pool);
//
//		// unbond with punish: 12.5Ring
//		assert_ok!(Staking::unbond_with_punish(
//			Origin::signed(controller_2),
//			125 * MILLICENTS / 10,
//			promise_month * MONTH_IN_SECONDS as u64
//		));
//		// unbond deposit items: 12.5Ring
//		Timestamp::set_timestamp(promise_month * MONTH_IN_SECONDS as u64);
//		assert_ok!(Staking::unbond(
//			Origin::signed(controller_2),
//			StakingBalance::Ring(125 * MILLICENTS / 10)
//		));
//		ring_pool -= 25 * MILLICENTS;
//		assert_eq!(Staking::ring_pool(), ring_pool);
//
//		// slash: 25Ring 50Kton
//		Staking::slash_validator(&stash_1, 1_000_000_000);
//		Staking::slash_validator(&stash_2, 1_000_000_000);
//		ring_pool -= 25 * MILLICENTS;
//		kton_pool -= 50 * MILLICENTS;
//		assert_eq!(Staking::ring_pool(), ring_pool);
//		assert_eq!(Staking::kton_pool(), kton_pool);
//	});
//}
//
//#[test]
//fn unbond_over_max_unbondings_chunks_should_fail() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		gen_paired_account!(stash(123), controller(456), promise_month(12));
//		let deposit_items_len = MAX_UNLOCKING_CHUNKS + 1;
//
//		for _ in 1..deposit_items_len {
//			assert_ok!(Staking::bond_extra(
//				Origin::signed(stash),
//				StakingBalance::Ring(MILLICENTS),
//				promise_month
//			));
//		}
//		{
//			let ledger = Staking::ledger(&controller).unwrap();
//			assert_eq!(ledger.deposit_items.len(), deposit_items_len);
//			assert_eq!(ledger.unbondings.len(), 0);
//		}
//
//		Timestamp::set_timestamp(promise_month as u64 * MONTH_IN_SECONDS as u64);
//
//		for _ in 1..deposit_items_len {
//			assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Ring(MILLICENTS)));
//		}
//		{
//			let ledger = Staking::ledger(&controller).unwrap();
//			assert_eq!(ledger.deposit_items.len(), 1);
//			assert_eq!(ledger.unbondings.len(), deposit_items_len - 1);
//		}
//		assert_err!(
//			Staking::unbond(
//				Origin::signed(controller),
//				StakingBalance::Ring((deposit_items_len - 1) as u64 * MILLICENTS)
//			),
//			"can not schedule more unlock chunks"
//		);
//	});
//}
//
//#[test]
//fn unlock_value_should_be_increased_and_decreased_correctly() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		// normal Ring/Kton
//		{
//			let stash = 444;
//			let controller = 555;
//			let _ = Ring::deposit_creating(&stash, 100 * MILLICENTS);
//			Kton::deposit_creating(&stash, 100 * MILLICENTS);
//
//			assert_ok!(Staking::bond(
//				Origin::signed(stash),
//				controller,
//				StakingBalance::Ring(50 * MILLICENTS),
//				RewardDestination::Stash,
//				0
//			));
//			assert_ok!(Staking::bond_extra(
//				Origin::signed(stash),
//				StakingBalance::Kton(50 * MILLICENTS),
//				0
//			));
//
//			let mut unbondings = Staking::ledger(&controller).unwrap().unbondings;
//
//			assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Ring(MILLICENTS)));
//			unbondings.push(NormalLock {
//				value: StakingBalance::Ring(MILLICENTS),
//				era: 3,
//				is_time_deposit: false,
//			});
//			assert_eq!(&Staking::ledger(&controller).unwrap().unbondings, &unbondings);
//			assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Kton(MILLICENTS)));
//			unbondings.push(NormalLock {
//				value: StakingBalance::Kton(MILLICENTS),
//				era: 3,
//				is_time_deposit: false,
//			});
//			assert_eq!(&Staking::ledger(&controller).unwrap().unbondings, &unbondings);
//
//			assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Ring(0)));
//			unbondings.push(NormalLock {
//				value: StakingBalance::Ring(0),
//				era: 3,
//				is_time_deposit: true,
//			});
//			assert_eq!(&Staking::ledger(&controller).unwrap().unbondings, &unbondings);
//			assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Kton(0)));
//			unbondings.push(NormalLock {
//				value: StakingBalance::Kton(0),
//				era: 3,
//				is_time_deposit: false,
//			});
//			assert_eq!(&Staking::ledger(&controller).unwrap().unbondings, &unbondings);
//		}
//
//		// promise Ring
//		{
//			gen_paired_account!(stash(666), controller(777), promise_month(12));
//
//			assert_ok!(Staking::bond_extra(
//				Origin::signed(stash),
//				StakingBalance::Ring(50 * MILLICENTS),
//				36
//			));
//
//			let mut unbondings = Staking::ledger(&controller).unwrap().unbondings;
//
//			assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Ring(MILLICENTS)));
//			unbondings.push(NormalLock {
//				value: StakingBalance::Ring(0),
//				era: 3,
//				is_time_deposit: true,
//			});
//			assert_eq!(&Staking::ledger(&controller).unwrap().unbondings, &unbondings);
//
//			for month in [12, 36].iter() {
//				assert_ok!(Staking::unbond_with_punish(
//					Origin::signed(controller),
//					20 * MILLICENTS,
//					month * MONTH_IN_SECONDS as u64
//				));
//				unbondings.push(NormalLock {
//					value: StakingBalance::Ring(20 * MILLICENTS),
//					era: 3,
//					is_time_deposit: true,
//				});
//				assert_eq!(&Staking::ledger(&controller).unwrap().unbondings, &unbondings);
//
//				assert_ok!(Staking::unbond_with_punish(
//					Origin::signed(controller),
//					29 * MILLICENTS,
//					month * MONTH_IN_SECONDS as u64
//				));
//				unbondings.push(NormalLock {
//					value: StakingBalance::Ring(29 * MILLICENTS),
//					era: 3,
//					is_time_deposit: true,
//				});
//				assert_eq!(&Staking::ledger(&controller).unwrap().unbondings, &unbondings);
//
//				assert_ok!(Staking::unbond_with_punish(
//					Origin::signed(controller),
//					50 * MILLICENTS,
//					month * MONTH_IN_SECONDS as u64
//				));
//				unbondings.push(NormalLock {
//					value: StakingBalance::Ring(1 * MILLICENTS),
//					era: 3,
//					is_time_deposit: true,
//				});
//				assert_eq!(&Staking::ledger(&controller).unwrap().unbondings, &unbondings);
//			}
//		}
//	});
//}
//
//// #[test]
//// fn total_deposit_should_be_increased_and_decreased_correctly() {
//// with_externalities(
//// &mut ExtBuilder::default().existential_deposit(0).build(),
//// || body,
//// );
//// }
//
//#[test]
//fn promise_extra_should_not_remove_unexpired_items() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		gen_paired_account!(stash(123), controller(456), promise_month(12));
//
//		let expired_item_len = 3;
//		let expiry_date = promise_month as u64 * MONTH_IN_SECONDS as u64;
//
//		assert_ok!(Staking::bond_extra(
//			Origin::signed(stash),
//			StakingBalance::Ring(5 * MILLICENTS),
//			0
//		));
//		for _ in 0..expired_item_len {
//			assert_ok!(Staking::promise_extra(Origin::signed(controller), MILLICENTS, promise_month));
//		}
//
//		Timestamp::set_timestamp(expiry_date - 1);
//		assert_ok!(Staking::promise_extra(
//			Origin::signed(controller),
//			2 * MILLICENTS,
//			promise_month
//		));
//		assert_eq!(
//			Staking::ledger(&controller).unwrap().deposit_items.len(),
//			2 + expired_item_len
//		);
//
//		Timestamp::set_timestamp(expiry_date);
//		assert_ok!(Staking::promise_extra(
//			Origin::signed(controller),
//			2 * MILLICENTS,
//			promise_month
//		));
//		assert_eq!(Staking::ledger(&controller).unwrap().deposit_items.len(), 2);
//	});
//}
//
//#[test]
//fn unbond_zero_before_expiry() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let expiry_date = 12 * MONTH_IN_SECONDS as u64;
//		let unbond_value = StakingBalance::Ring(MILLICENTS);
//
//		Timestamp::set_timestamp(expiry_date - 1);
//		assert_ok!(Staking::unbond(Origin::signed(10), unbond_value.clone()));
//		assert_eq!(
//			Staking::ledger(&10).unwrap().unbondings[0].value,
//			StakingBalance::Ring(0)
//		);
//
//		Timestamp::set_timestamp(expiry_date);
//		assert_ok!(Staking::unbond(Origin::signed(10), unbond_value.clone()));
//		assert_eq!(Staking::ledger(&10).unwrap().unbondings[1].value, unbond_value);
//	});
//}
//
//// bond 10_000 Ring for 12 months, gain 1 Kton
//// bond extra 10_000 Ring for 36 months, gain 3 Kton
//// bond extra 1 Kton
//// nominate
//// unlock the 12 months deposit item with punish
//// lost 3 Kton and 10_000 Ring's power for nominate
//#[test]
//fn yakio_q1() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let stash = 777;
//		let controller = 888;
//		let _ = Ring::deposit_creating(&stash, 20_000);
//
//		assert_ok!(Staking::bond(
//			Origin::signed(stash),
//			controller,
//			StakingBalance::Ring(10_000),
//			RewardDestination::Stash,
//			12
//		));
//		assert_ok!(Staking::bond_extra(
//			Origin::signed(stash),
//			StakingBalance::Ring(10_000),
//			36
//		));
//		assert_eq!(Kton::free_balance(&stash), 4);
//
//		assert_ok!(Staking::bond_extra(Origin::signed(stash), StakingBalance::Kton(1), 36));
//		assert_eq!(Staking::ledger(&controller).unwrap().active_kton, 1);
//
//		assert_ok!(Staking::nominate(Origin::signed(controller), vec![controller]));
//
//		assert_ok!(Staking::unbond_with_punish(
//			Origin::signed(controller),
//			10_000 * MILLICENTS,
//			12 * MONTH_IN_SECONDS as u64
//		));
//		assert_eq!(Kton::free_balance(&stash), 1);
//
//		let ledger = StakingLedger {
//			stash: 777,
//			total_deposit_ring: 10_000,
//			active_ring: 10_000,
//			active_deposit_ring: 10_000,
//			active_kton: 1,
//			deposit_items: vec![TimeDepositItem {
//				value: 10_000,
//				start_time: 0,
//				expire_time: 36 * MONTH_IN_SECONDS as u64,
//			}],
//			unbondings: vec![],
//		};
//		start_era(3);
//		assert_ok!(Staking::withdraw_unbonded(Origin::signed(controller)));
//		assert_eq!(&Staking::ledger(&controller).unwrap(), &ledger);
//		// not enough Kton to unbond
//		assert_ok!(Staking::unbond_with_punish(
//			Origin::signed(controller),
//			10_000 * MILLICENTS,
//			36 * MONTH_IN_SECONDS as u64
//		));
//		assert_eq!(&Staking::ledger(&controller).unwrap(), &ledger);
//	});
//}
//
//// how to balance the power and calculate the reward if some validators have been chilled
//#[test]
//fn yakio_q2() {
//	fn run(with_new_era: bool) -> u64 {
//		let mut balance = 0;
//		ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//			gen_paired_account!(validator_1_stash(123), validator_1_controller(456), 0);
//			gen_paired_account!(validator_2_stash(234), validator_2_controller(567), 0);
//			gen_paired_account!(nominator_stash(345), nominator_controller(678), 0);
//
//			assert_ok!(Staking::validate(
//				Origin::signed(validator_1_controller),
//				vec![0; 8],
//				0,
//				3
//			));
//			assert_ok!(Staking::validate(
//				Origin::signed(validator_2_controller),
//				vec![1; 8],
//				0,
//				3
//			));
//			assert_ok!(Staking::nominate(
//				Origin::signed(nominator_controller),
//				vec![validator_1_stash, validator_2_stash]
//			));
//
//			start_era(1);
//			assert_ok!(Staking::chill(Origin::signed(validator_1_controller)));
//			// assert_ok!(Staking::chill(Origin::signed(validator_2_controller)));
//			if with_new_era {
//				start_era(2);
//			}
//			Staking::reward_validator(&validator_1_stash, 1000 * MILLICENTS);
//			Staking::reward_validator(&validator_2_stash, 1000 * MILLICENTS);
//
//			balance = Ring::free_balance(&nominator_stash);
//		});
//
//		balance
//	}
//
//	let free_balance = run(false);
//	let free_balance_with_new_era = run(true);
//
//	assert_ne!(free_balance, 0);
//	assert_ne!(free_balance_with_new_era, 0);
//	assert!(free_balance > free_balance_with_new_era);
//}

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
				reasons: WithdrawReasons::all()
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
				reasons: WithdrawReasons::all()
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
				reasons: WithdrawReasons::all()
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
				reasons: WithdrawReasons::all()
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
				ring_staking_lock: StakingLock {
					staking_amount: 0,
					unbondings: vec![]
				},
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
				reasons: WithdrawReasons::all()
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
				reasons: WithdrawReasons::all()
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
				reasons: WithdrawReasons::all()
			}]
		);
		//		println!("Ok Unbond - Ring Balance: {:?}", Ring::free_balance(stash));
		//		println!("Ok Unbond - Ring Locks: {:#?}", Ring::locks(stash));
		//		println!();

		assert_err!(
			Ring::transfer(Origin::signed(stash), controller, 1),
			"account liquidity restrictions prevent withdrawal"
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
				reasons: WithdrawReasons::all()
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
				reasons: WithdrawReasons::all()
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
				kton_staking_lock: StakingLock {
					staking_amount: 0,
					unbondings: vec![]
				},
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
				reasons: WithdrawReasons::all()
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
				reasons: WithdrawReasons::all()
			}]
		);
		//		println!("Ok Bond Extra - Kton Balance: {:?}", Kton::free_balance(stash));
		//		println!("Ok Bond Extra - Kton Locks: {:#?}", Kton::locks(stash));
		//		println!();

		let (unbond_value_1, unbond_start_1) = (2, 2);
		Timestamp::set_timestamp(unbond_start_1);
		assert_ok!(Staking::unbond(
			Origin::signed(controller),
			StakingBalance::Kton(unbond_value_1)
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
				reasons: WithdrawReasons::all()
			}]
		);
		//		println!("Ok Unbond - Kton Balance: {:?}", Kton::free_balance(stash));
		//		println!("Ok Unbond - Kton Locks: {:#?}", Kton::locks(stash));
		//		println!();

		let (unbond_value_2, unbond_start_2) = (6, 3);
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
				reasons: WithdrawReasons::all()
			}]
		);
		//		println!("Ok Unbond - Kton Balance: {:?}", Kton::free_balance(stash));
		//		println!("Ok Unbond - Kton Locks: {:#?}", Kton::locks(stash));
		//		println!();

		assert_err!(
			Kton::transfer(Origin::signed(stash), controller, unbond_value_1),
			"account liquidity restrictions prevent withdrawal"
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
			"account liquidity restrictions prevent withdrawal"
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
				reasons: WithdrawReasons::all()
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
				reasons: WithdrawReasons::all()
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
				reasons: WithdrawReasons::all()
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
				reasons: WithdrawReasons::all()
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
				reasons: WithdrawReasons::all()
			}]
		);
		//		println!("Ok Bond Extra - Ring Balance: {:?}", Ring::free_balance(stash));
		//		println!("Ok Bond Extra - Ring Locks: {:#?}", Ring::locks(stash));
		//		println!();

		let (unbond_value_1, unbond_start_1) = (2, 2);
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
				reasons: WithdrawReasons::all()
			}]
		);
		//		println!("Ok Unbond - Ring Balance: {:?}", Ring::free_balance(stash));
		//		println!("Ok Unbond - Ring Locks: {:#?}", Ring::locks(stash));
		//		println!();

		let (unbond_value_2, unbond_start_2) = (6, 3);
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
				reasons: WithdrawReasons::all()
			}]
		);
		//		println!("Ok Unbond - Ring Balance: {:?}", Ring::free_balance(stash));
		//		println!("Ok Unbond - Ring Locks: {:#?}", Ring::locks(stash));
		//		println!();

		assert_err!(
			Ring::transfer(Origin::signed(stash), controller, unbond_value_1),
			"account liquidity restrictions prevent withdrawal"
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
			"account liquidity restrictions prevent withdrawal"
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
				reasons: WithdrawReasons::all()
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
				reasons: WithdrawReasons::all()
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
				reasons: WithdrawReasons::all()
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
				ring_staking_lock: StakingLock {
					staking_amount: 0,
					unbondings: vec![]
				},
				kton_staking_lock: StakingLock {
					staking_amount: 5,
					unbondings: vec![]
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
				ring_staking_lock: StakingLock {
					staking_amount: 0,
					unbondings: vec![]
				},
				kton_staking_lock: StakingLock {
					staking_amount: 0,
					unbondings: vec![NormalLock { amount: 5, until: 61 }]
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
				ring_staking_lock: StakingLock {
					staking_amount: 0,
					unbondings: vec![]
				},
				kton_staking_lock: StakingLock {
					staking_amount: 1,
					unbondings: vec![NormalLock { amount: 5, until: 61 }]
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
					unbondings: vec![]
				},
				kton_staking_lock: StakingLock {
					staking_amount: 0,
					unbondings: vec![]
				},
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
					unbondings: vec![NormalLock { amount: 5, until: 61 }]
				},
				kton_staking_lock: StakingLock {
					staking_amount: 0,
					unbondings: vec![]
				},
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
					unbondings: vec![NormalLock { amount: 5, until: 61 }]
				},
				kton_staking_lock: StakingLock {
					staking_amount: 0,
					unbondings: vec![]
				},
			}
		);
		//		println!("Locks: {:#?}", Ring::locks(stash));
		//		println!("StakingLedger: {:#?}", Staking::ledger(&controller));
		//		println!();
	});
}
