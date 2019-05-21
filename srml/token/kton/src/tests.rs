#![cfg(test)]

use super::*;
use runtime_io::with_externalities;
use srml_support::{assert_ok, assert_noop, assert_err};
use mock::{Ring, System, Timestamp, Kton, Test, ExtBuilder, Origin};
use srml_support::traits::Currency;
use ring::BalanceLock;


#[test]
fn ext_builer_should_work() {
    // test existential_deposit setting
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(0).build(),
        || {
            assert_eq!(Ring::free_balance(&1), 10);
        });

    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(1).build(),
                       || {
                           assert_eq!(Ring::free_balance(&1), 10000);
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
        .build(),
        || {
            Kton::deposit(Origin::signed(100), 10000, 12);

            let kton_balance = Kton::free_balance(&100);
            assert_eq!(kton_balance, 1);

            Kton::deposit(Origin::signed(101), 100000, 36);
            assert_eq!( Kton::free_balance(&101), 36);
        });
}

#[test]
fn check_deposit_status() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(1).build(),
        || {
            // the initial free_balance of 11 is 100;
            Kton::deposit(Origin::signed(11), 100000, 36);
            // ensure locked ring can not be withdrew
            // 900001 = 1000k - 100k
            assert_eq!( Kton::free_balance(&11), 36);
            // lock liquidity
            assert_noop!(Ring::transfer(Origin::signed(11), 1001, 900001), "account liquidity restrictions prevent withdrawal");

            let now = Timestamp::now();
            // check deposit info
            assert_eq!(Kton::deposit_ledger(&11), Some(Deposit {
                total_deposit: 100000,
                deposit_list: vec![DepositInfo {
                                    month: 36,
                                    start_at: now,
                                    value: 100000,
                                    unit_interest: 0,
                                    claimed: false,
                }]
            }));

            // check ring locks
            assert_eq!(Ring::locks(&11), vec![ring::BalanceLock {id: DEPOSIT_ID, amount: 100000_u64, until: u64::max_value(), reasons: WithdrawReasons::all()}]);

            Kton::deposit(Origin::signed(11), 200000, 36);
            assert_eq!( Kton::free_balance(&11), 108);
            assert_eq!(Kton::total_issuance(), 108);

            assert_eq!(Kton::deposit_ledger(&11), Some(Deposit {
                total_deposit: 300000,
                deposit_list: vec![
                    DepositInfo {
                        month: 36,
                        start_at: now,
                        value: 100000,
                        unit_interest: 0,
                        claimed: false,
                },
                    DepositInfo {
                        month: 36,
                        start_at: now,
                        value: 200000,
                        unit_interest: 0,
                        claimed: false,
                    }]
            }));

            // check ring locks
            assert_eq!(Ring::locks(&11), vec![ring::BalanceLock {id: DEPOSIT_ID, amount: 300000_u64, until: u64::max_value(), reasons: WithdrawReasons::all()}]);


        });


}