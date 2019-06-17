#![cfg(test)]

use mock::{ExtBuilder, Kton, Origin, Ring, System, Test, Timestamp};
use ring::BalanceLock;
use runtime_io::with_externalities;
use srml_support::{assert_err, assert_noop, assert_ok};
use srml_support::traits::{ Currency, Imbalance };
use evo_support::traits::SystemCurrency;

use super::*;

#[inline]
fn compute_dividend_of(acc: u64) -> i128 {
    let kton_balance = Kton::free_balance(&acc) as u64;
    let paid_out = Kton::reward_paidout(&acc);
    let reward_per_share = Kton::reward_per_share() as u64;
    let should_withdraw = i128::from(reward_per_share * kton_balance) - paid_out;
    should_withdraw
}

#[inline]
fn set_reward_per_share_hundred(acc: u64) {
    Kton::deposit(Origin::signed(acc), 100000 * DECIMALS, 36);
    // now acc has 36 unit kton
    // 360 of 1200 flow into ktoner pool
    Kton::transfer_to_system(Origin::signed(101), 3600 * DECIMALS);
    // reward_per_share = 3600 / 36 = 100
    assert_eq!(Kton::reward_per_share(), 100 * DECIMALS);
}


#[test]
fn ext_builer_should_work() {
    // test existential_deposit setting
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(0).build(), || {
        assert_eq!(Ring::free_balance(&1), 10 * DECIMALS);
    });

    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(1).build(),
                       || {
                           assert_eq!(Ring::free_balance(&1), 10000 * DECIMALS);
                       });
}

#[test]
fn check_sys_account() {
    with_externalities(&mut ExtBuilder::default()
        .build(),
                       || {
                           let sys_account = Kton::sys_account();
                           assert_eq!(sys_account, 42_u64);
                       });
}


#[test]
fn check_deposit_ring_related_balance() {
    with_externalities(&mut ExtBuilder::default()
        .build(), || {
        Kton::deposit(Origin::signed(100), 10000 * DECIMALS, 12);

        let kton_balance = Kton::free_balance(&100);
        assert_eq!(kton_balance, 1 * DECIMALS);

        Kton::deposit(Origin::signed(101), 100000 * DECIMALS, 36);
        assert_eq!(Kton::free_balance(&101), 36 * DECIMALS);
    });
}

//#[test]
//fn check_deposit_status() {
//    with_externalities(&mut ExtBuilder::default()
//        .existential_deposit(1).build(), || {
//        // the initial free_balance of 11 is 100;
//        Kton::deposit(Origin::signed(11), 100000, 36);
//        // ensure locked ring can not be withdrew
//        // 900001 = 1000k - 100k
//        assert_eq!(Kton::free_balance(&11), 36);
//        // lock liquidity
//        assert_noop!(Ring::transfer(Origin::signed(11), 1001, 900001), "account liquidity restrictions prevent withdrawal");
//
//        let now = Timestamp::now();
//        // check deposit info
//        assert_eq!(Kton::deposit_ledger(&11), Some(Deposit {
//            total_deposit: 100000,
//            deposit_list: vec![DepositInfo {
//                month: 36,
//                start_at: now,
//                value: 100000,
//                unit_interest: 0,
//                claimed: false,
//            }],
//        }));
//
//        // check ring locks
//        assert_eq!(Ring::locks(&11), vec![ring::BalanceLock { id: DEPOSIT_ID, amount: 100000_u64, until: u64::max_value(), reasons: WithdrawReasons::all() }]);
//
//        Kton::deposit(Origin::signed(11), 200000, 36);
//        assert_eq!(Kton::free_balance(&11), 108);
//        assert_eq!(Kton::total_issuance(), 108);
//
//        assert_eq!(Kton::deposit_ledger(&11), Some(Deposit {
//            total_deposit: 300000,
//            deposit_list: vec![
//                DepositInfo {
//                    month: 36,
//                    start_at: now,
//                    value: 100000,
//                    unit_interest: 0,
//                    claimed: false,
//                },
//                DepositInfo {
//                    month: 36,
//                    start_at: now,
//                    value: 200000,
//                    unit_interest: 0,
//                    claimed: false,
//                }],
//        }));
//
//        // check ring locks
//        assert_eq!(Ring::locks(&11), vec![ring::BalanceLock { id: DEPOSIT_ID, amount: 300000_u64, until: u64::max_value(), reasons: WithdrawReasons::all() }]);
//    },
//    );
//}
//
//#[test]
//fn check_reward_per_share() {
//    with_externalities(&mut ExtBuilder::default()
//        .existential_deposit(1).build(), || {
//        Kton::deposit(Origin::signed(11), 100000, 36);
//        // now acc 11 has 36 unit kton
//        // 360 of 1200 flow into ktoner pool
//        Kton::transfer_to_system(Origin::signed(101), 360);
//        // reward_per_share = 360 / 36 = 10
//        assert_eq!(Kton::reward_per_share(), 10);
//        // kton total_issurance = 72
//        // kton_balance of acc 101 is 36
//        Kton::deposit(Origin::signed(101), 100000, 36);
//        // 720 of 2400 flow into ktoner pool
//        Kton::transfer_to_system(Origin::signed(11), 720);
//        // reward_per_share = 10 + 720 / 72 = 20
//        assert_eq!(Kton::reward_per_share(), 20);
//        // old_price * new_balance = 10 * 36 = 360
//        assert_eq!(Kton::reward_paidout(&101), 360);
//
//        // acc 11 should withdraw 360 ring as reward
//        assert_eq!(compute_dividend_of(101), 360_i128);
//
//        Kton::transfer(Origin::signed(101), 2, 36);
//        assert_eq!(Kton::free_balance(&101), 0);
//        assert_eq!(Kton::free_balance(&2), 36);
//        // after transfer, reward stick to these ktons
//        // still belongs to acc 11, not acc 2
//        assert_eq!(compute_dividend_of(101), 360_i128);
//        assert_eq!(compute_dividend_of(2), 0_i128);
//    });
//}
//
//#[test]
//fn check_acc_claim_reward() {
//    with_externalities(&mut ExtBuilder::default()
//        .existential_deposit(1).build(), || {
//
//        let ring_total_issuance = Ring::total_issuance();
//        set_reward_per_share_hundred(11);
//        // now reward_per_share is 100
//        // acc 11 has 1 kton
//        assert_eq!(compute_dividend_of(11), 3600);
//        // no change on total_issuance of ring
//        assert_eq!(Ring::total_issuance(), ring_total_issuance);
//        // air drop 1000 ring to acc 101
//        // note: -1000
//        <RewardPaidOut<Test>>::insert(&101, -1000);
//        let ring_balance_acc_101 = Ring::free_balance(&101);
//        assert_eq!(compute_dividend_of(101), 1000);
//
//        Kton::claim_reward(Origin::signed(101));
//        assert_eq!(Ring::free_balance(&101), ring_balance_acc_101);
//
//        // air drop 10000 ring to acc 12
//        <RewardPaidOut<Test>>::insert(&12, -10000);
//        let ring_balance_acc_12 = Ring::free_balance(&12);
//        assert_eq!(compute_dividend_of(12), 10000);
//        Kton::claim_reward(Origin::signed(12));
//        assert_eq!(Ring::free_balance(&12), ring_balance_acc_12 + 9000);
//    });
//}
//
//
