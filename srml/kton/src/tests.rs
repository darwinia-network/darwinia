use srml_support::{assert_err, assert_noop, assert_ok, traits::Currency};

use darwinia_support::{LockIdentifier, NormalLock, TimeStamp, WithdrawLock, WithdrawReason, WithdrawReasons};

use super::*;
use crate::mock::*;

const ID_1: LockIdentifier = *b"1       ";
const ID_2: LockIdentifier = *b"2       ";
const ID_3: LockIdentifier = *b"3       ";

#[test]
fn transfer_should_work() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		let _ = Kton::deposit_creating(&666, 100);

		assert_ok!(Kton::transfer(Origin::signed(666), 777, 50));
		assert_eq!(Kton::total_balance(&666), 50);
		assert_eq!(Kton::total_balance(&777), 50);

		assert_ok!(Kton::transfer(Origin::signed(666), 777, 50));
		assert_eq!(Kton::total_balance(&666), 0);
		assert_eq!(Kton::total_balance(&777), 100);

		assert_ok!(Kton::transfer(Origin::signed(666), 777, 0));
	});
}

// TODO
#[test]
fn transfer_should_fail() {
	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
		let _ = Kton::deposit_creating(&777, 1);
		assert_err!(
			Kton::transfer(Origin::signed(666), 777, 50),
			"balance too low to send value",
		);

		let _ = Kton::deposit_creating(&666, Balance::max_value());
		assert_err!(
			Kton::transfer(Origin::signed(777), 666, 1),
			"destination balance too high to receive value",
		);

		assert_err!(
			Kton::transfer(Origin::signed(1), 777, Kton::vesting_balance(&1)),
			"vesting balance too high to send value",
		);

		Kton::set_lock(
			ID_1,
			&777,
			WithdrawLock::Normal(NormalLock {
				amount: Balance::max_value(),
				until: TimeStamp::max_value(),
			}),
			WithdrawReasons::all(),
		);
		assert_err!(
			Kton::transfer(Origin::signed(777), 1, 1),
			"account liquidity restrictions prevent withdrawal",
		);
	});
}

//#[test]
//fn set_lock_should_work() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let lock_ids = [[0; 8], [1; 8], [2; 8], [3; 8]];
//		let balance_per_lock = Kton::free_balance(&1) / (lock_ids.len() as u64);
//
//		// account `1`'s vesting length
//		System::set_block_number(4);
//
//		{
//			let mut locks = vec![];
//			for lock_id in lock_ids.iter() {
//				Kton::set_lock(*lock_id, &1, balance_per_lock, u64::max_value(), WithdrawReasons::all());
//				locks.push(BalanceLock {
//					id: *lock_id,
//					amount: balance_per_lock,
//					until: u64::max_value(),
//					reasons: WithdrawReasons::all(),
//				});
//				assert_eq!(Kton::locks(&1), locks);
//			}
//		}
//
//		for _ in 0..lock_ids.len() - 1 {
//			assert_ok!(Kton::transfer(Origin::signed(1), 2, balance_per_lock));
//		}
//		assert_err!(
//			Kton::transfer(Origin::signed(1), 2, balance_per_lock),
//			"account liquidity restrictions prevent withdrawal"
//		);
//	});
//}
//
//#[test]
//fn remove_lock_should_work() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		Timestamp::set_timestamp(0);
//		let ts: u64 = Timestamp::now().into();
//
//		Kton::set_lock(ID_1, &2, u64::max_value(), u64::max_value(), WithdrawReasons::all());
//		assert_err!(
//			Kton::transfer(Origin::signed(2), 1, 1),
//			"account liquidity restrictions prevent withdrawal"
//		);
//
//		// unexpired
//		Kton::set_lock(ID_2, &2, u64::max_value(), ts + 1, WithdrawReasons::all());
//		Kton::remove_lock(ID_1, &2);
//		Timestamp::set_timestamp(ts);
//		assert_err!(
//			Kton::transfer(Origin::signed(2), 1, 1),
//			"account liquidity restrictions prevent withdrawal"
//		);
//		Kton::remove_lock(ID_2, &2);
//		assert_ok!(Kton::transfer(Origin::signed(2), 1, 1));
//
//		// expired
//		Kton::set_lock(ID_3, &2, u64::max_value(), ts, WithdrawReasons::all());
//		assert_ok!(Kton::transfer(Origin::signed(2), 1, 1));
//	});
//}
//
//#[test]
//fn update_lock_should_work() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		let mut locks = vec![];
//		for id in 0..10 {
//			// until > 1
//			locks.push(BalanceLock {
//				id: [id; 8],
//				amount: 1,
//				until: 2,
//				reasons: WithdrawReasons::none(),
//			});
//			Kton::set_lock([id; 8], &1, 1, 2, WithdrawReasons::none());
//		}
//		let update_id = 4;
//		for amount in 32767u64..65535 {
//			let until = amount + 1;
//			locks[update_id as usize] = BalanceLock {
//				id: [update_id; 8],
//				amount,
//				until,
//				reasons: WithdrawReasons::all(),
//			};
//			Kton::set_lock([update_id; 8], &1, amount, until, WithdrawReasons::all());
//			assert_eq!(Kton::locks(&1), locks);
//		}
//	});
//}
//
//#[test]
//fn combination_locking_should_work() {
//	ExtBuilder::default().existential_deposit(0).build().execute_with(|| {
//		Kton::deposit_creating(&1001, 10);
//		Kton::set_lock(ID_1, &1001, u64::max_value(), 0, WithdrawReasons::none());
//		Kton::set_lock(ID_2, &1001, 0, u64::max_value(), WithdrawReasons::none());
//		Kton::set_lock(ID_3, &1001, 0, 0, WithdrawReasons::all());
//		assert_ok!(Kton::transfer(Origin::signed(1001), 1002, 1));
//	});
//}
