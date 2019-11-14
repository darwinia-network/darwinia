use srml_support::{
	assert_err, assert_ok,
	traits::{Currency, WithdrawReason, WithdrawReasons},
};

use super::MONTH_IN_SECONDS;
use super::*;
use crate::mock::*;

// gen_paired_account!(a(1), b(2), m(12));
// will create stash `a` and controller `b`
// `a` has 100 Ring and 100 Kton
// promise for `m` month with 50 Ring and 50 Kton
// `m` can be ignore, and it wont perfrom `bond` action
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
			$how_long
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
			$how_long
			));
		assert_ok!(Staking::bond_extra(
			Origin::signed($stash),
			StakingBalance::Kton(50 * COIN),
			$how_long
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

		assert_eq!(Staking::bonded(&11), Some(10));
		assert_eq!(
			Staking::ledger(&10),
			Some(StakingLedgers {
				stash: 11,
				total_ring: 100 * COIN,
				total_deposit_ring: 100 * COIN,
				active_deposit_ring: 100 * COIN,
				active_ring: 100 * COIN,
				total_kton: 0,
				active_kton: 0,
				deposit_items: vec![TimeDepositItem {
					value: 100 * COIN,
					start_time: 0,
					expire_time: 12 * MONTH_IN_SECONDS as u64
				}],
				unlocking_count: 0
			})
		);

		assert_eq!(Kton::free_balance(&11), COIN / 100);
		assert_eq!(Kton::total_issuance(), 16 * COIN / 100);

		let origin_ledger = Staking::ledger(&10).unwrap();
		let _ = Ring::deposit_creating(&11, 100 * COIN);
		assert_ok!(Staking::bond_extra(
			Origin::signed(11),
			StakingBalance::Ring(20 * COIN),
			13
		));
		assert_eq!(
			Staking::ledger(&10),
			Some(StakingLedgers {
				stash: 11,
				total_ring: origin_ledger.total_ring + 20 * COIN,
				total_deposit_ring: origin_ledger.total_deposit_ring + 20 * COIN,
				active_deposit_ring: origin_ledger.active_deposit_ring + 20 * COIN,
				active_ring: origin_ledger.active_ring + 20 * COIN,
				total_kton: 0,
				active_kton: 0,
				deposit_items: vec![
					TimeDepositItem {
						value: 100 * COIN,
						start_time: 0,
						expire_time: 12 * MONTH_IN_SECONDS as u64
					},
					TimeDepositItem {
						value: 20 * COIN,
						start_time: 0,
						expire_time: 13 * MONTH_IN_SECONDS as u64
					}
				],
				unlocking_count: 0
			})
		);
	});
}

//#[test]
//fn normal_kton_should_work() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		Kton::deposit_creating(&1001, 10 * COIN);
//		assert_ok!(Staking::bond(
//			Origin::signed(1001),
//			1000,
//			StakingBalance::Kton(10 * COIN),
//			RewardDestination::Stash,
//			0
//		));
//		assert_eq!(
//			Staking::ledger(&1000),
//			Some(StakingLedgers {
//				stash: 1001,
//				total_ring: 0,
//				total_deposit_ring: 0,
//				active_deposit_ring: 0,
//				active_ring: 0,
//				total_kton: 10 * COIN,
//				active_kton: 10 * COIN,
//				deposit_items: vec![],
//				unlocking: vec![]
//			})
//		);
//
//		assert_eq!(
//			Kton::locks(&1001),
//			vec![kton::BalanceLock {
//				id: STAKING_ID,
//				amount: 10 * COIN,
//				until: u64::max_value(),
//				reasons: WithdrawReasons::all()
//			}]
//		);
//
//		// promise_month should not work for kton
//		Kton::deposit_creating(&2001, 10 * COIN);
//		assert_ok!(Staking::bond(
//			Origin::signed(2001),
//			2000,
//			StakingBalance::Kton(10 * COIN),
//			RewardDestination::Stash,
//			12
//		));
//		assert_eq!(
//			Staking::ledger(&2000),
//			Some(StakingLedgers {
//				stash: 2001,
//				total_ring: 0,
//				total_deposit_ring: 0,
//				active_deposit_ring: 0,
//				active_ring: 0,
//				total_kton: 10 * COIN,
//				active_kton: 10 * COIN,
//				deposit_items: vec![],
//				unlocking: vec![]
//			})
//		);
//	});
//}
//
//#[test]
//fn time_deposit_ring_unbond_and_withdraw_should_work() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		Timestamp::set_timestamp(13 * MONTH_IN_SECONDS as u64);
//
//		assert_ok!(Staking::unbond(Origin::signed(10), StakingBalance::Ring(10 * COIN)));
//		assert_eq!(
//			Staking::ledger(&10),
//			Some(StakingLedgers {
//				stash: 11,
//				total_ring: 100 * COIN,
//				total_deposit_ring: 100 * COIN,
//				active_deposit_ring: 90 * COIN,
//				active_ring: 90 * COIN,
//				total_kton: 0,
//				active_kton: 0,
//				deposit_items: vec![TimeDepositItem {
//					value: 90 * COIN,
//					start_time: 0,
//					expire_time: 12 * MONTH_IN_SECONDS as u64
//				}],
//				unlocking: vec![UnlockChunk {
//					value: StakingBalance::Ring(10 * COIN),
//					era: 3,
//					is_time_deposit: true
//				}]
//			})
//		);
//
//		assert_ok!(Staking::unbond(Origin::signed(10), StakingBalance::Ring(20 * COIN)));
//		assert_eq!(
//			Staking::ledger(&10),
//			Some(StakingLedgers {
//				stash: 11,
//				total_ring: 100 * COIN,
//				total_deposit_ring: 100 * COIN,
//				active_deposit_ring: 70 * COIN,
//				active_ring: 70 * COIN,
//				total_kton: 0,
//				active_kton: 0,
//				deposit_items: vec![TimeDepositItem {
//					value: 70 * COIN,
//					start_time: 0,
//					expire_time: 12 * MONTH_IN_SECONDS as u64
//				}],
//				unlocking: vec![
//					UnlockChunk {
//						value: StakingBalance::Ring(10 * COIN),
//						era: 3,
//						is_time_deposit: true
//					},
//					UnlockChunk {
//						value: StakingBalance::Ring(20 * COIN),
//						era: 3,
//						is_time_deposit: true
//					}
//				]
//			})
//		);
//
//		// more than active ring
//		assert_ok!(Staking::unbond(Origin::signed(10), StakingBalance::Ring(120 * COIN)));
//		assert_eq!(
//			Staking::ledger(&10),
//			Some(StakingLedgers {
//				stash: 11,
//				total_ring: 100 * COIN,
//				total_deposit_ring: 100 * COIN,
//				active_deposit_ring: 0,
//				active_ring: 0,
//				total_kton: 0,
//				active_kton: 0,
//				deposit_items: vec![], // should be cleared
//				unlocking: vec![
//					UnlockChunk {
//						value: StakingBalance::Ring(10 * COIN),
//						era: 3,
//						is_time_deposit: true
//					},
//					UnlockChunk {
//						value: StakingBalance::Ring(20 * COIN),
//						era: 3,
//						is_time_deposit: true
//					},
//					UnlockChunk {
//						value: StakingBalance::Ring(70 * COIN),
//						era: 3,
//						is_time_deposit: true
//					},
//				]
//			})
//		);
//
//		start_era(3);
//
//		assert_ok!(Staking::withdraw_unbonded(Origin::signed(10)));
//		assert_eq!(
//			Staking::ledger(&10),
//			Some(StakingLedgers {
//				stash: 11,
//				total_ring: 0,
//				total_deposit_ring: 0,
//				active_deposit_ring: 0,
//				active_ring: 0,
//				total_kton: 0,
//				active_kton: 0,
//				deposit_items: vec![], // should be cleared
//				unlocking: vec![]
//			})
//		);
//
//		let free_balance = Ring::free_balance(&11);
//		assert_eq!(
//			Ring::locks(&11),
//			vec![balances::BalanceLock {
//				id: STAKING_ID,
//				amount: 0,
//				until: u64::max_value(),
//				reasons: WithdrawReasons::all()
//			}]
//		);
//		assert_ok!(Ring::ensure_can_withdraw(
//			&11,
//			free_balance,
//			WithdrawReason::Transfer,
//			0
//		));
//	});
//}
//
//#[test]
//fn normal_unbond_should_work() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let stash = 11;
//		let controller = 10;
//		let value = 200 * COIN;
//		let promise_month = 12;
//		// unbond normal ring
//		let _ = Ring::deposit_creating(&stash, 1000 * COIN);
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
//				kton_free_balance + utils::compute_kton_return::<Test>(value, promise_month)
//			);
//			ledger.total_ring += value;
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
//				StakingBalance::Kton(COIN),
//				0
//			));
//			ledger.total_kton += kton_free_balance;
//			ledger.active_kton += kton_free_balance;
//			assert_eq!(&Staking::ledger(&controller).unwrap(), &ledger);
//
//			assert_ok!(Staking::unbond(
//				Origin::signed(controller),
//				StakingBalance::Kton(kton_free_balance)
//			));
//			ledger.active_kton = 0;
//			ledger.unlocking = vec![UnlockChunk {
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
//		let _ = Ring::deposit_creating(&stash, 100 * COIN);
//		Kton::deposit_creating(&stash, COIN / 100000);
//
//		// timestamp now is 0.
//		// free balance of kton is too low to work
//		assert_ok!(Staking::bond(
//			Origin::signed(stash),
//			controller,
//			StakingBalance::Ring(10 * COIN),
//			RewardDestination::Stash,
//			promise_month
//		));
//		assert_eq!(
//			Staking::ledger(&controller),
//			Some(StakingLedgers {
//				stash,
//				total_ring: 10 * COIN,
//				total_deposit_ring: 10 * COIN,
//				active_deposit_ring: 10 * COIN,
//				active_ring: 10 * COIN,
//				total_kton: 0,
//				active_kton: 0,
//				deposit_items: vec![TimeDepositItem {
//					value: 10 * COIN,
//					start_time: 0,
//					expire_time: promise_month as u64 * MONTH_IN_SECONDS as u64
//				}], // should be cleared
//				unlocking: vec![]
//			})
//		);
//		let mut ledger = Staking::ledger(&controller).unwrap();
//		let kton_free_balance = Kton::free_balance(&stash);
//		// kton is 0, skip unbond_with_punish
//		assert_ok!(Staking::unbond_with_punish(
//			Origin::signed(controller),
//			10 * COIN,
//			promise_month as u64 * MONTH_IN_SECONDS as u64
//		));
//		assert_eq!(&Staking::ledger(&controller).unwrap(), &ledger);
//		assert_eq!(Kton::free_balance(&stash), kton_free_balance);
//
//		// set more kton balance to make it work
//		Kton::deposit_creating(&stash, 10 * COIN);
//		let kton_free_balance = Kton::free_balance(&stash);
//		let unbond_value = 5 * COIN;
//		assert_ok!(Staking::unbond_with_punish(
//			Origin::signed(controller),
//			unbond_value,
//			promise_month as u64 * MONTH_IN_SECONDS as u64
//		));
//		ledger.active_ring -= unbond_value;
//		ledger.active_deposit_ring -= unbond_value;
//		ledger.deposit_items[0].value -= unbond_value;
//		ledger.unlocking = vec![UnlockChunk {
//			value: StakingBalance::Ring(unbond_value),
//			era: 3,
//			is_time_deposit: true,
//		}];
//		assert_eq!(&Staking::ledger(&controller).unwrap(), &ledger);
//
//		let kton_punishment = utils::compute_kton_return::<Test>(unbond_value, promise_month);
//		assert_eq!(Kton::free_balance(&stash), kton_free_balance - 3 * kton_punishment);
//
//		// if deposit_item.value == 0
//		// the whole item should be be dropped
//		assert_ok!(Staking::unbond_with_punish(
//			Origin::signed(controller),
//			5 * COIN,
//			promise_month as u64 * MONTH_IN_SECONDS as u64
//		));
//		assert!(Staking::ledger(&controller).unwrap().deposit_items.is_empty());
//	});
//}
//
//#[test]
//fn transform_to_promised_ring_should_work() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let _ = Ring::deposit_creating(&1001, 100 * COIN);
//		assert_ok!(Staking::bond(
//			Origin::signed(1001),
//			1000,
//			StakingBalance::Ring(10 * COIN),
//			RewardDestination::Stash,
//			0
//		));
//		let origin_ledger = Staking::ledger(&1000).unwrap();
//		let kton_free_balance = Kton::free_balance(&1001);
//
//		assert_ok!(Staking::promise_extra(Origin::signed(1000), 5 * COIN, 12));
//
//		assert_eq!(
//			Staking::ledger(&1000),
//			Some(StakingLedgers {
//				stash: 1001,
//				total_ring: origin_ledger.total_ring,
//				total_deposit_ring: origin_ledger.total_deposit_ring + 5 * COIN,
//				active_deposit_ring: origin_ledger.active_deposit_ring + 5 * COIN,
//				active_ring: origin_ledger.active_ring,
//				total_kton: origin_ledger.total_kton,
//				active_kton: origin_ledger.active_kton,
//				deposit_items: vec![TimeDepositItem {
//					value: 5 * COIN,
//					start_time: 0,
//					expire_time: 12 * MONTH_IN_SECONDS as u64
//				}],
//				unlocking: vec![]
//			})
//		);
//
//		assert_eq!(Kton::free_balance(&1001), kton_free_balance + (5 * COIN / 10000));
//	});
//}
//
//#[test]
//fn expired_ring_should_capable_to_promise_again() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let _ = Ring::deposit_creating(&1001, 100 * COIN);
//		assert_ok!(Staking::bond(
//			Origin::signed(1001),
//			1000,
//			StakingBalance::Ring(10 * COIN),
//			RewardDestination::Stash,
//			12
//		));
//		let mut ledger = Staking::ledger(&1000).unwrap();
//		let ts = 13 * MONTH_IN_SECONDS as u64;
//		let promise_extra_value = 5 * COIN;
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
//#[test]
//fn inflation_should_be_correct() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let initial_issuance = 1_200_000_000 * COIN;
//		let surplus_needed = initial_issuance - Ring::total_issuance();
//		let _ = Ring::deposit_into_existing(&11, surplus_needed);
//		assert_eq!(Ring::total_issuance(), initial_issuance);
//		assert_eq!(Staking::current_era_total_reward(), 80000000 * COIN / 10);
//		start_era(11);
//		// ErasPerEpoch = 10
//		assert_eq!(Staking::current_era_total_reward(), 88000000 * COIN / 10);
//	});
//}
//
//#[test]
//fn reward_should_work_correctly() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		// create controller account
//		let _ = Ring::deposit_creating(&2000, COIN);
//		let _ = Ring::deposit_creating(&1000, COIN);
//		let _ = Ring::deposit_creating(&200, COIN);
//		// new validator
//		let _ = Ring::deposit_creating(&2001, 2000 * COIN);
//		Kton::deposit_creating(&2001, 10 * COIN);
//		// new validator
//		let _ = Ring::deposit_creating(&1001, 300 * COIN);
//		Kton::deposit_creating(&1001, 1 * COIN);
//		// handle some dirty work
//		let _ = Ring::deposit_creating(&201, 2000 * COIN);
//		Kton::deposit_creating(&201, 10 * COIN);
//		assert_eq!(Kton::free_balance(&201), 10 * COIN);
//
//		// 2001-2000
//		assert_ok!(Staking::bond(
//			Origin::signed(2001),
//			2000,
//			StakingBalance::Ring(300 * COIN),
//			RewardDestination::Controller,
//			12,
//		));
//		assert_ok!(Staking::bond_extra(
//			Origin::signed(2001),
//			StakingBalance::Kton(1 * COIN),
//			0
//		));
//		// 1001-1000
//		assert_ok!(Staking::bond(
//			Origin::signed(1001),
//			1000,
//			StakingBalance::Ring(300 * COIN),
//			RewardDestination::Controller,
//			12,
//		));
//		assert_ok!(Staking::bond_extra(
//			Origin::signed(1001),
//			StakingBalance::Kton(1 * COIN),
//			0
//		));
//		let ring_pool = Staking::ring_pool();
//		let kton_pool = Staking::kton_pool();
//		// 201-200
//		assert_ok!(Staking::bond(
//			Origin::signed(201),
//			200,
//			StakingBalance::Ring(3000 * COIN - ring_pool),
//			RewardDestination::Stash,
//			12,
//		));
//		assert_ok!(Staking::bond_extra(
//			Origin::signed(201),
//			StakingBalance::Kton(10 * COIN - kton_pool),
//			0,
//		));
//		// ring_pool and kton_pool
//		assert_eq!(Staking::ring_pool(), 3000 * COIN);
//		assert_eq!(Staking::kton_pool(), 10 * COIN);
//		// 1/5 ring_pool and 1/5 kton_pool
//		assert_ok!(Staking::validate(Origin::signed(2000), [0; 8].to_vec(), 0, 3));
//		assert_ok!(Staking::nominate(Origin::signed(1000), vec![2001]));
//
//		assert_eq!(Staking::ledger(&2000).unwrap().active_kton, 1 * COIN);
//		assert_eq!(Staking::ledger(&2000).unwrap().active_ring, 300 * COIN);
//		assert_eq!(Staking::slashable_balance_of(&2001), 600 * COIN as u128);
//		// 600COIN for rewarding ring bond-er
//		// 600COIN for rewarding kton bond-er
//		Staking::select_validators();
//		Staking::reward_validator(&2001, 1200 * COIN);
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
//		assert_eq!(Ring::free_balance(&2000), 601 * COIN);
//		assert_eq!(Ring::free_balance(&1000), 601 * COIN);
//	});
//}
//
//#[test]
//fn slash_should_work() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let _ = Ring::deposit_creating(&1001, 100 * COIN);
//		Kton::deposit_creating(&1001, 100 * COIN);
//
//		assert_ok!(Staking::bond(
//			Origin::signed(1001),
//			1000,
//			StakingBalance::Ring(50 * COIN),
//			RewardDestination::Controller,
//			0,
//		));
//		assert_ok!(Staking::bond_extra(
//			Origin::signed(1001),
//			StakingBalance::Kton(50 * COIN),
//			0
//		));
//		assert_ok!(Staking::validate(Origin::signed(1000), [0; 8].to_vec(), 0, 3));
//
//		// slash 1%
//		let slash_value = 5 * COIN / 10;
//		let mut ledger = Staking::ledger(&1000).unwrap();
//		let ring_free_balance = Ring::free_balance(&1001);
//		let kton_free_balance = Kton::free_balance(&1001);
//		Staking::slash_validator(&1001, 10_000_000);
//		ledger.total_ring -= slash_value;
//		ledger.active_ring -= slash_value;
//		ledger.total_kton -= slash_value;
//		ledger.active_kton -= slash_value;
//		assert_eq!(&Staking::ledger(&1000).unwrap(), &ledger);
//		assert_eq!(Ring::free_balance(&1001), ring_free_balance - slash_value);
//		assert_eq!(Kton::free_balance(&1001), kton_free_balance - slash_value);
//	});
//}
//
//#[test]
//fn test_inflation() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		assert_eq!(Staking::current_era_total_reward(), 80_000_000 * COIN / 10);
//		start_era(20);
//		assert_eq!(Staking::epoch_index(), 2);
//		assert_eq!(Staking::current_era_total_reward(), 9_999_988_266 * COIN / 1000);
//	});
//}
//
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
//		assert_eq!(Staking::ledger(&10).unwrap().total_ring, 100 * COIN);
//		assert_ok!(Staking::set_controller(Origin::signed(11), 12));
//		assert_eq!(Staking::ledger(&12).unwrap().total_ring, 100 * COIN);
//	});
//}
//
//#[test]
//fn slash_should_not_touch_unlockings() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let old_ledger = Staking::ledger(&10).unwrap();
//		// only deposit_ring, no normal_ring
//		assert_eq!(
//			(
//				old_ledger.total_ring,
//				old_ledger.active_ring,
//				old_ledger.active_deposit_ring
//			),
//			(100 * COIN, 100 * COIN, 100 * COIN)
//		);
//
//		assert_ok!(Staking::bond_extra(
//			Origin::signed(11),
//			StakingBalance::Ring(100 * COIN),
//			0
//		));
//		Kton::deposit_creating(&11, 10 * COIN);
//		assert_ok!(Staking::bond_extra(
//			Origin::signed(11),
//			StakingBalance::Kton(10 * COIN),
//			0
//		));
//
//		assert_ok!(Staking::unbond(Origin::signed(10), StakingBalance::Ring(10 * COIN)));
//		let new_ledger = Staking::ledger(&10).unwrap();
//		assert_eq!(
//			(
//				new_ledger.total_ring,
//				new_ledger.active_ring,
//				new_ledger.active_deposit_ring
//			),
//			(200 * COIN, 190 * COIN, 100 * COIN)
//		);
//
//		// slash 100%
//		Staking::slash_validator(&11, 1_000_000_000);
//
//		let ledger = Staking::ledger(&10).unwrap();
//		assert_eq!(
//			(ledger.total_ring, ledger.active_ring, ledger.active_deposit_ring),
//			// 10Ring in unlocking
//			(10 * COIN, 0, 0)
//		);
//		assert_eq!(ledger.unlocking[0].value, StakingBalance::Ring(10 * COIN));
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
//				StakingBalance::Ring(COIN),
//				RewardDestination::Stash,
//				37
//			),
//			"months at most is 36."
//		);
//
//		gen_paired_account!(stash(123), controller(456), promise_month(12));
//		assert_err!(
//			Staking::bond_extra(Origin::signed(stash), StakingBalance::Ring(COIN), 37),
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
//				StakingBalance::Ring(COIN),
//				RewardDestination::Stash,
//				0
//			),
//			"stash already bonded"
//		);
//		assert_err!(
//			Staking::bond(
//				Origin::signed(unpaired_stash),
//				10,
//				StakingBalance::Ring(COIN),
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
//		ring_pool += 100 * COIN;
//		kton_pool += 100 * COIN;
//		assert_eq!(Staking::ring_pool(), ring_pool);
//		assert_eq!(Staking::kton_pool(), kton_pool);
//
//		// unbond: 50Ring 50Kton
//		assert_ok!(Staking::unbond(
//			Origin::signed(controller_1),
//			StakingBalance::Ring(50 * COIN)
//		));
//		assert_ok!(Staking::unbond(
//			Origin::signed(controller_1),
//			StakingBalance::Kton(25 * COIN)
//		));
//		// not yet expired: promise for 12 months
//		assert_ok!(Staking::unbond(
//			Origin::signed(controller_2),
//			StakingBalance::Ring(50 * COIN)
//		));
//		assert_ok!(Staking::unbond(
//			Origin::signed(controller_2),
//			StakingBalance::Kton(25 * COIN)
//		));
//		ring_pool -= 50 * COIN;
//		kton_pool -= 50 * COIN;
//		assert_eq!(Staking::ring_pool(), ring_pool);
//		assert_eq!(Staking::kton_pool(), kton_pool);
//
//		// unbond with punish: 12.5Ring
//		assert_ok!(Staking::unbond_with_punish(
//			Origin::signed(controller_2),
//			125 * COIN / 10,
//			promise_month * MONTH_IN_SECONDS as u64
//		));
//		// unbond deposit items: 12.5Ring
//		Timestamp::set_timestamp(promise_month * MONTH_IN_SECONDS as u64);
//		assert_ok!(Staking::unbond(
//			Origin::signed(controller_2),
//			StakingBalance::Ring(125 * COIN / 10)
//		));
//		ring_pool -= 25 * COIN;
//		assert_eq!(Staking::ring_pool(), ring_pool);
//
//		// slash: 25Ring 50Kton
//		Staking::slash_validator(&stash_1, 1_000_000_000);
//		Staking::slash_validator(&stash_2, 1_000_000_000);
//		ring_pool -= 25 * COIN;
//		kton_pool -= 50 * COIN;
//		assert_eq!(Staking::ring_pool(), ring_pool);
//		assert_eq!(Staking::kton_pool(), kton_pool);
//	});
//}
//
//#[test]
//fn unbond_over_max_unlocking_chunks_should_fail() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		gen_paired_account!(stash(123), controller(456), promise_month(12));
//		let deposit_items_len = MAX_UNLOCKING_CHUNKS + 1;
//
//		for _ in 1..deposit_items_len {
//			assert_ok!(Staking::bond_extra(
//				Origin::signed(stash),
//				StakingBalance::Ring(COIN),
//				promise_month
//			));
//		}
//		{
//			let ledger = Staking::ledger(&controller).unwrap();
//			assert_eq!(ledger.deposit_items.len(), deposit_items_len);
//			assert_eq!(ledger.unlocking.len(), 0);
//		}
//
//		Timestamp::set_timestamp(promise_month as u64 * MONTH_IN_SECONDS as u64);
//
//		for _ in 1..deposit_items_len {
//			assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Ring(COIN)));
//		}
//		{
//			let ledger = Staking::ledger(&controller).unwrap();
//			assert_eq!(ledger.deposit_items.len(), 1);
//			assert_eq!(ledger.unlocking.len(), deposit_items_len - 1);
//		}
//		assert_err!(
//			Staking::unbond(
//				Origin::signed(controller),
//				StakingBalance::Ring((deposit_items_len - 1) as u64 * COIN)
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
//			let _ = Ring::deposit_creating(&stash, 100 * COIN);
//			Kton::deposit_creating(&stash, 100 * COIN);
//
//			assert_ok!(Staking::bond(
//				Origin::signed(stash),
//				controller,
//				StakingBalance::Ring(50 * COIN),
//				RewardDestination::Stash,
//				0
//			));
//			assert_ok!(Staking::bond_extra(
//				Origin::signed(stash),
//				StakingBalance::Kton(50 * COIN),
//				0
//			));
//
//			let mut unlocking = Staking::ledger(&controller).unwrap().unlocking;
//
//			assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Ring(COIN)));
//			unlocking.push(UnlockChunk {
//				value: StakingBalance::Ring(COIN),
//				era: 3,
//				is_time_deposit: false,
//			});
//			assert_eq!(&Staking::ledger(&controller).unwrap().unlocking, &unlocking);
//			assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Kton(COIN)));
//			unlocking.push(UnlockChunk {
//				value: StakingBalance::Kton(COIN),
//				era: 3,
//				is_time_deposit: false,
//			});
//			assert_eq!(&Staking::ledger(&controller).unwrap().unlocking, &unlocking);
//
//			assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Ring(0)));
//			unlocking.push(UnlockChunk {
//				value: StakingBalance::Ring(0),
//				era: 3,
//				is_time_deposit: true,
//			});
//			assert_eq!(&Staking::ledger(&controller).unwrap().unlocking, &unlocking);
//			assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Kton(0)));
//			unlocking.push(UnlockChunk {
//				value: StakingBalance::Kton(0),
//				era: 3,
//				is_time_deposit: false,
//			});
//			assert_eq!(&Staking::ledger(&controller).unwrap().unlocking, &unlocking);
//		}
//
//		// promise Ring
//		{
//			gen_paired_account!(stash(666), controller(777), promise_month(12));
//
//			assert_ok!(Staking::bond_extra(
//				Origin::signed(stash),
//				StakingBalance::Ring(50 * COIN),
//				36
//			));
//
//			let mut unlocking = Staking::ledger(&controller).unwrap().unlocking;
//
//			assert_ok!(Staking::unbond(Origin::signed(controller), StakingBalance::Ring(COIN)));
//			unlocking.push(UnlockChunk {
//				value: StakingBalance::Ring(0),
//				era: 3,
//				is_time_deposit: true,
//			});
//			assert_eq!(&Staking::ledger(&controller).unwrap().unlocking, &unlocking);
//
//			for month in [12, 36].iter() {
//				assert_ok!(Staking::unbond_with_punish(
//					Origin::signed(controller),
//					20 * COIN,
//					month * MONTH_IN_SECONDS as u64
//				));
//				unlocking.push(UnlockChunk {
//					value: StakingBalance::Ring(20 * COIN),
//					era: 3,
//					is_time_deposit: true,
//				});
//				assert_eq!(&Staking::ledger(&controller).unwrap().unlocking, &unlocking);
//
//				assert_ok!(Staking::unbond_with_punish(
//					Origin::signed(controller),
//					29 * COIN,
//					month * MONTH_IN_SECONDS as u64
//				));
//				unlocking.push(UnlockChunk {
//					value: StakingBalance::Ring(29 * COIN),
//					era: 3,
//					is_time_deposit: true,
//				});
//				assert_eq!(&Staking::ledger(&controller).unwrap().unlocking, &unlocking);
//
//				assert_ok!(Staking::unbond_with_punish(
//					Origin::signed(controller),
//					50 * COIN,
//					month * MONTH_IN_SECONDS as u64
//				));
//				unlocking.push(UnlockChunk {
//					value: StakingBalance::Ring(1 * COIN),
//					era: 3,
//					is_time_deposit: true,
//				});
//				assert_eq!(&Staking::ledger(&controller).unwrap().unlocking, &unlocking);
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
//			StakingBalance::Ring(5 * COIN),
//			0
//		));
//		for _ in 0..expired_item_len {
//			assert_ok!(Staking::promise_extra(Origin::signed(controller), COIN, promise_month));
//		}
//
//		Timestamp::set_timestamp(expiry_date - 1);
//		assert_ok!(Staking::promise_extra(
//			Origin::signed(controller),
//			2 * COIN,
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
//			2 * COIN,
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
//		let unbond_value = StakingBalance::Ring(COIN);
//
//		Timestamp::set_timestamp(expiry_date - 1);
//		assert_ok!(Staking::unbond(Origin::signed(10), unbond_value.clone()));
//		assert_eq!(
//			Staking::ledger(&10).unwrap().unlocking[0].value,
//			StakingBalance::Ring(0)
//		);
//
//		Timestamp::set_timestamp(expiry_date);
//		assert_ok!(Staking::unbond(Origin::signed(10), unbond_value.clone()));
//		assert_eq!(Staking::ledger(&10).unwrap().unlocking[1].value, unbond_value);
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
//		assert_eq!(Staking::ledger(&controller).unwrap().total_kton, 1);
//
//		assert_ok!(Staking::nominate(Origin::signed(controller), vec![controller]));
//
//		assert_ok!(Staking::unbond_with_punish(
//			Origin::signed(controller),
//			10_000 * COIN,
//			12 * MONTH_IN_SECONDS as u64
//		));
//		assert_eq!(Kton::free_balance(&stash), 1);
//
//		let ledger = StakingLedgers {
//			stash: 777,
//			total_ring: 10_000,
//			total_deposit_ring: 10_000,
//			active_ring: 10_000,
//			active_deposit_ring: 10_000,
//			total_kton: 1,
//			active_kton: 1,
//			deposit_items: vec![TimeDepositItem {
//				value: 10_000,
//				start_time: 0,
//				expire_time: 36 * MONTH_IN_SECONDS as u64,
//			}],
//			unlocking: vec![],
//		};
//		start_era(3);
//		assert_ok!(Staking::withdraw_unbonded(Origin::signed(controller)));
//		assert_eq!(&Staking::ledger(&controller).unwrap(), &ledger);
//		// not enough Kton to unbond
//		assert_ok!(Staking::unbond_with_punish(
//			Origin::signed(controller),
//			10_000 * COIN,
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
//			Staking::reward_validator(&validator_1_stash, 1000 * COIN);
//			Staking::reward_validator(&validator_2_stash, 1000 * COIN);
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
		gen_paired_account!(stash(123), controller(456), promise_month(12));
		assert_ok!(Staking::bond_extra(
			Origin::signed(stash),
			StakingBalance::Ring(50 * COIN),
			24
		));
		println!("Ledger: {:#?}", Staking::ledger(controller));
		println!("Stash Free Balance: {:?}", Kton::free_balance(stash));
		println!("Stash Kton Locks: {:?}", Kton::locks(stash));

		let ts = 13 * MONTH_IN_SECONDS as u64;
		Timestamp::set_timestamp(ts);

		assert_ok!(Staking::unbond(
			Origin::signed(controller),
			StakingBalance::Kton(50 * COIN)
		));

		println!("Stash Kton Locks: {:?}", Kton::locks(stash));
	});
}
