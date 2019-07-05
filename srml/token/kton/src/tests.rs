#![cfg(test)]

use balances::BalanceLock;
use mock::{ExtBuilder, Kton, Origin, Ring, System, Test, Timestamp};
use mock::DECIMALS;
use runtime_io::with_externalities;
use srml_support::{assert_err, assert_noop, assert_ok};
use srml_support::traits::{Currency, Imbalance};

use super::*;

#[inline]
fn approximate_equal(real: u64, ideal: u64) -> bool {
    (real - ideal) * 100 / ideal < 2
}


#[inline]
fn deposit_pre() {
    Kton::deposit(Origin::signed(11), 100000, 12);
    Kton::deposit(Origin::signed(21), 100000, 36);
}

#[inline]
fn deposit_with_decimals_pre() {
    // acc deposit 100w ring
    Kton::deposit(Origin::signed(11), 10_000_000_000 * DECIMALS, 12);
    Kton::deposit(Origin::signed(21), 1_000_000_000 * DECIMALS, 36);
}

#[test]
fn ext_builer_should_work() {
    // test existential_deposit setting
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(0).build(), || {
        assert_eq!(Ring::free_balance(&1), 10 * DECIMALS);
    });

    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(1).build(), || {
        assert_eq!(Ring::free_balance(&1), 10000 * DECIMALS);
    });
}


#[test]
fn deposit_and_deposit_extra_should_work() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(1).build(), || {
        assert_eq!(Kton::free_balance(&11), 0);
        deposit_pre();
        assert_eq!(Kton::free_balance(&11), 10);
        assert_eq!(Kton::free_balance(&21), 36);

        Kton::deposit_extra(Origin::signed(11), 10000, 12);
        assert_eq!(Kton::free_balance(&11), 11);
    });
}


#[test]
fn deposit_and_deposit_extra_with_decimals_should_work() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(1).build(), || {
        assert_eq!(Kton::free_balance(&11), 0);
        deposit_with_decimals_pre();
        assert_eq!(Kton::free_balance(&11), 1000000 * DECIMALS);
        assert!(approximate_equal(Kton::free_balance(&21), 360000 * DECIMALS));
    });
}