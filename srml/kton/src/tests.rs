#![cfg(test)]

use balances::BalanceLock;
use mock::{ExtBuilder, Kton, Origin, Ring, System, Test, Timestamp};
use runtime_io::with_externalities;
use srml_support::{assert_err, assert_noop, assert_ok};
use srml_support::traits::{Currency, Imbalance};
use node_runtime::{ MILLI, COIN };

use super::*;

#[inline]
fn approximate_equal(real: u128, ideal: u128) -> bool {
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
    Kton::deposit(Origin::signed(11), 10_000_000_000 * COIN, 12);
    Kton::deposit(Origin::signed(21), 1_000_000_000 * COIN, 36);
}

#[test]
fn ext_builer_should_work() {
    // test existential_deposit setting
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(0).build(), || {
        assert_eq!(Ring::free_balance(&1), 10 * COIN);
    });

    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(1).build(), || {
        assert_eq!(Ring::free_balance(&1), 10000 * COIN);
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
        assert_err!(Kton::deposit(Origin::signed(11), 10000, 12), "Already deposited.");
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
        assert_eq!(Kton::free_balance(&11), 1000000 * COIN);
        assert!(approximate_equal(Kton::free_balance(&21), 360000 * COIN));
    });
}

#[inline]
fn reward_per_share_not_zero() {
    // new acc 91, 92, 93 got 100k ring
    assert_ok!(Ring::transfer(Origin::signed(11), 91, 100_000 * COIN));
    assert_ok!(Ring::transfer(Origin::signed(11), 92, 100_000 * COIN));
    assert_ok!(Ring::transfer(Origin::signed(11), 93, 100_000 * COIN));

    // acc 91 and 92 deposit 10k ring for 12 months
    // in return, acc 91 and 92 will get 1 kton
    Kton::deposit(Origin::signed(91), 10_000 * COIN, 12);
    Kton::deposit(Origin::signed(92), 10_000 * COIN, 12);
    assert_eq!(Kton::total_issuance(), 2 * COIN);

    Kton::reward_to_pot(6000 * COIN);
    assert_eq!(Kton::reward_per_share(), 3000);

}

#[test]
fn reward_per_share_should_work() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(MILLI).build(), || {
        // reward_per_share 3000
        reward_per_share_not_zero();

        assert_eq!(Kton::reward_per_share(), 3000);
        assert_eq!(Kton::free_balance(&91), 1 * COIN);

        // acc 91 and 92 can withdraw 3k ring as reward
        assert_eq!(Kton::reward_can_withdraw(&91), 3000 * COIN);
        assert_eq!(Kton::reward_can_withdraw(&91), 3000 * COIN);

        Kton::deposit(Origin::signed(93), 10_000 * COIN, 12);
        // acc 93 has got 1 kton and reward_per_share is 3000
        assert_eq!(Kton::reward_paid_out(&93), 3000 * COIN as i128);
        // after acc 93 has got kton
        // there is no system revenue
        // so acc 93 should withdraw 0
        assert_eq!(Kton::reward_can_withdraw(&93), 0);

    });
}

#[test]
fn transfer_should_work() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(MILLI).build(), || {
        // reward_per_share 3000
        reward_per_share_not_zero();
        Kton::deposit(Origin::signed(93), 10_000 * COIN, 12);
        // acc 93 has got 1 kton and reward_per_share is 3000
        assert_eq!(Kton::reward_paid_out(&93), 3000 * COIN as i128);
        assert_eq!(Kton::reward_can_withdraw(&93), 0);

        // new things happen!
        // reward_per_share now change to
        Kton::reward_to_pot(3000 * COIN);
        assert_eq!(Ring::free_balance(&Kton::sys_acc()), 9000 * COIN);
        assert_eq!(Kton::reward_per_share(), 4000);
        assert_eq!(Kton::reward_can_withdraw(&93), 1000 * COIN);

        // before transfer:
        // acc 93 has 1 kton and can withdraw 1000 ring
        // acc 94 has 0 kton and can withdraw 0 ring
        // after tranfer:
        // acc 93 has 0 kton and can withdraw 1000 ring
        // acc 94 has 1 kton and can withdraw 0 ring
        assert_eq!(Kton::free_balance(&93), 1 * COIN);
        assert_eq!(Kton::free_balance(&94), 0);
        assert_eq!(Kton::reward_can_withdraw(&93), 1000 * COIN);
        assert_eq!(Kton::reward_can_withdraw(&94), 0);

        assert_eq!(Kton::reward_paid_out(&93), 3000 * COIN as i128);
        Kton::transfer(Origin::signed(93), 94, 1 * COIN);
        assert_eq!(Kton::reward_paid_out(&93), -1000 * COIN as i128);

        assert_eq!(Kton::free_balance(&93), 0);
        assert_eq!(Kton::free_balance(&94), 1 * COIN);
        assert_eq!(Kton::reward_can_withdraw(&93), 1000 * COIN);
        assert_eq!(Kton::reward_can_withdraw(&94), 0);

    });
}

#[test]
fn withdraw_reward_should_work() {

    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(MILLI).build(), || {
        // reward_per_share 3000
        // acc 91 and 92 have 1 kton
        reward_per_share_not_zero();

        assert_eq!(Kton::reward_can_withdraw(&91), 3000 * COIN);
        let old_91_free_balance = Ring::free_balance(&91);
        let old_sys_free_balance = Ring::free_balance(&Kton::sys_acc());
        Kton::claim_reward(Origin::signed(91));
        assert_eq!(Kton::reward_can_withdraw(&91), 0);
        assert_eq!(Ring::free_balance(&91), old_91_free_balance + 3000 * COIN);
        assert_eq!(Ring::free_balance(&Kton::sys_acc()), old_sys_free_balance - 3000 * COIN);
    });
}

#[test]
fn make_free_balance_be_should_work() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(MILLI).build(), || {
        // reward_per_share 3000
        // acc 91 and 92 have 1 kton
        reward_per_share_not_zero();
        // before:
        // acc 94 has 0 kton and 0 reward_paid_out
        // after:
        // acc 94 has 1 kton and 3k reward_paid_out
        // total_issuance + 1
        let old_total_issuance = Kton::total_issuance();
        Kton::make_free_balance_be(&94, 1 * COIN);
        assert_eq!(Kton::free_balance(&94), 1 * COIN);
        assert_eq!(Kton::reward_paid_out(&94), 3000 * COIN as i128);
        assert_eq!(Kton::reward_can_withdraw(&94), 0);
        assert_eq!(Kton::total_issuance(), old_total_issuance + 1 * COIN);

        // before:
        // acc 91 has 1 kton and 3k reward_paid_out
        // after:
        // acc 91 has 0 kton and 3k rewrd_paid_out
        // total_issuance - 1
        let old_total_issuance = Kton::total_issuance();
        assert_eq!(Kton::reward_paid_out(&91), 0 as i128);
        assert_eq!(Kton::reward_can_withdraw(&91), 3000 * COIN);

        Kton::make_free_balance_be(&91, 0);
        assert_eq!(Kton::free_balance(&91), 0);
        assert_eq!(Kton::reward_paid_out(&91), -3000 * COIN as i128);
        assert_eq!(Kton::reward_can_withdraw(&91), 3000 * COIN);
        assert_eq!(Kton::total_issuance(), old_total_issuance - 1 * COIN);

    });
}


