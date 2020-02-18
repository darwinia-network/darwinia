use frame_support::{assert_err, assert_noop, assert_ok, traits::Currency};

use crate::{mock::*, *};
use darwinia_support::{LockIdentifier, NormalLock, WithdrawLock, WithdrawReasons};

const ID_1: LockIdentifier = *b"1       ";
const ID_2: LockIdentifier = *b"2       ";
const ID_3: LockIdentifier = *b"3       ";

#[cfg(feature = "with-fee")]
mod with_transfer_fee {
	use super::*;

	#[test]
	fn transfer_should_work() {
		ExtBuilder::default().build().execute_with(|| {
			let _ = Kton::deposit_creating(&666, 100);

			assert_err!(
				Kton::transfer(Origin::signed(666), 777, 50),
				"Transfer Fee - NOT ENOUGH RING",
			);

			let _ = Ring::deposit_creating(&666, 1 * MICRO);
			assert_ok!(Kton::transfer(Origin::signed(666), 777, 50));
			assert_eq!(Kton::total_balance(&666), 50);
			assert_eq!(Kton::total_balance(&777), 50);

			assert_err!(
				Kton::transfer(Origin::signed(666), 777, 50),
				"Transfer Fee - NOT ENOUGH RING",
			);

			let _ = Ring::deposit_creating(&666, 1 * MICRO);
			assert_ok!(Kton::transfer(Origin::signed(666), 777, 50));
			assert_eq!(Kton::total_balance(&666), 0);
			assert_eq!(Kton::total_balance(&777), 100);
		});
	}
}

#[cfg(not(feature = "with-fee"))]
mod without_transfer_fee {
	use super::*;

	#[test]
	fn transfer_should_work() {
		ExtBuilder::default().build().execute_with(|| {
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

	#[test]
	fn transfer_should_fail() {
		ExtBuilder::default()
			.vesting(true)
			.monied(true)
			.build()
			.execute_with(|| {
				let _ = Kton::deposit_creating(&777, 1);
				assert_noop!(
					Kton::transfer(Origin::signed(666), 777, 50),
					Error::<Test>::InsufficientBalance,
				);

				let _ = Kton::deposit_creating(&666, Balance::max_value());
				assert_noop!(Kton::transfer(Origin::signed(777), 666, 1), Error::<Test>::Overflow);

				assert_err!(
					Kton::transfer(Origin::signed(2), 777, Kton::vesting_balance(&2)),
					Error::<Test>::VestingBalance,
				);

				Kton::set_lock(
					ID_1,
					&777,
					WithdrawLock::Normal(NormalLock {
						amount: Balance::max_value(),
						until: Moment::max_value(),
					}),
					WithdrawReasons::all(),
				);
				assert_noop!(
					Kton::transfer(Origin::signed(777), 1, 1),
					Error::<Test>::LiquidityRestrictions,
				);
			});
	}

	#[test]
	fn set_lock_should_work() {
		ExtBuilder::default().monied(true).build().execute_with(|| {
			let lock_ids = [[0; 8], [1; 8], [2; 8], [3; 8], [4; 8]];

			let balance_per_lock = Kton::free_balance(&1) / (lock_ids.len() as Balance);

			assert_eq!(balance_per_lock, 2);
			{
				let mut locks = vec![];
				for lock_id in lock_ids.iter() {
					Kton::set_lock(
						*lock_id,
						&1,
						WithdrawLock::Normal(NormalLock {
							amount: balance_per_lock,
							until: Moment::max_value(),
						}),
						WithdrawReasons::all(),
					);
					locks.push(BalanceLock {
						id: *lock_id,
						withdraw_lock: WithdrawLock::Normal(NormalLock {
							amount: balance_per_lock,
							until: Moment::max_value(),
						}),
						reasons: WithdrawReasons::all(),
					});
					assert_eq!(Kton::locks(&1), locks);
				}
			}

			for _ in 0..lock_ids.len() - 1 {
				assert_ok!(Kton::transfer(Origin::signed(1), 2, balance_per_lock));
			}
			assert_noop!(
				Kton::transfer(Origin::signed(1), 2, balance_per_lock),
				Error::<Test>::LiquidityRestrictions
			);
		});
	}

	// TODO: The timestamp is removed, use block number to check this
	#[test]
	fn lock_release_then_remove_lock_then_lock_again_should_work() {
		ExtBuilder::default().monied(true).build().execute_with(|| {
			System::set_block_number(1);
			// Lock 1 should released
			Kton::set_lock(
				ID_1,
				&2,
				WithdrawLock::Normal(NormalLock {
					amount: Balance::max_value(),
					until: 2,
				}),
				WithdrawReasons::all(),
			);
			assert_err!(
				Kton::transfer(Origin::signed(2), 1, 1),
				Error::<Test>::LiquidityRestrictions
			);
			System::set_block_number(2);
			assert_ok!(Kton::transfer(Origin::signed(2), 1, 1));

			// Lock 2 should be remove
			Kton::set_lock(
				ID_2,
				&2,
				WithdrawLock::Normal(NormalLock {
					amount: Balance::max_value(),
					until: 3,
				}),
				WithdrawReasons::all(),
			);
			assert_noop!(
				Kton::transfer(Origin::signed(2), 1, 1),
				Error::<Test>::LiquidityRestrictions
			);
			Kton::remove_lock(ID_2, &2);
			assert_ok!(Kton::transfer(Origin::signed(2), 1, 1));

			System::set_block_number(3);
			// Lock 3 should work after all lock released and removed
			Kton::set_lock(
				ID_3,
				&2,
				WithdrawLock::Normal(NormalLock {
					amount: Balance::max_value(),
					until: 4,
				}),
				WithdrawReasons::all(),
			);
			assert_noop!(
				Kton::transfer(Origin::signed(2), 1, 1),
				Error::<Test>::LiquidityRestrictions
			);
		});
	}

	#[test]
	fn update_lock_should_work() {
		ExtBuilder::default().build().execute_with(|| {
			let mut locks = vec![];
			for id in 0..10 {
				// until > 1
				locks.push(BalanceLock {
					id: [id; 8],
					withdraw_lock: WithdrawLock::Normal(NormalLock { amount: 1, until: 2 }),
					reasons: WithdrawReasons::none(),
				});
				Kton::set_lock(
					[id; 8],
					&1,
					WithdrawLock::Normal(NormalLock { amount: 1, until: 2 }),
					WithdrawReasons::none(),
				);
			}
			let update_id = 4;
			for amount in 32767..65535 {
				let until = amount as Moment + 1;
				locks[update_id as usize] = BalanceLock {
					id: [update_id; 8],
					withdraw_lock: WithdrawLock::Normal(NormalLock { amount, until }),
					reasons: WithdrawReasons::all(),
				};
				Kton::set_lock(
					[update_id; 8],
					&1,
					WithdrawLock::Normal(NormalLock { amount, until }),
					WithdrawReasons::all(),
				);
				assert_eq!(Kton::locks(&1), locks);
			}
		});
	}

	#[test]
	fn combination_locking_should_work() {
		ExtBuilder::default().build().execute_with(|| {
			let _ = Kton::deposit_creating(&1001, 10);
			Kton::set_lock(
				ID_1,
				&1001,
				WithdrawLock::Normal(NormalLock {
					amount: Balance::max_value(),
					until: 0,
				}),
				WithdrawReasons::none(),
			);
			Kton::set_lock(
				ID_2,
				&1001,
				WithdrawLock::Normal(NormalLock {
					amount: 0,
					until: Moment::max_value(),
				}),
				WithdrawReasons::none(),
			);
			Kton::set_lock(
				ID_3,
				&1001,
				WithdrawLock::Normal(NormalLock { amount: 0, until: 0 }),
				WithdrawReasons::all(),
			);
			assert_ok!(Kton::transfer(Origin::signed(1001), 1002, 1));
		});
	}
}
