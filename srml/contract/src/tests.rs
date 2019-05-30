#![cfg(test)]

use super::*;
use mock::{ExtBuilder, Kton, Origin, Ring, System, Test, Timestamp, Contract};
use runtime_io::with_externalities;
use srml_support::{assert_err, assert_noop, assert_ok};
use evo_support::traits::SystemCurrency;

// TODO: test total_issuance of ring before and after paying gas

#[inline]
fn set_reward_per_share_hundred(acc: u64) {
    Kton::deposit(Origin::signed(acc), 100000, 36);
    // now acc has 36 unit kton
    // 360 of 1200 flow into ktoner pool
    Kton::transfer_to_system(Origin::signed(101), 12000);
    // reward_per_share = 3600 / 36 = 100
    assert_eq!(Kton::reward_per_share(), 100);
}

#[test]
fn check_reward_per_share() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(1).build(), || {
        Kton::deposit(Origin::signed(11), 100000, 36);
        // now acc 11 has 36 unit kton
        // 360 of 1200 flow into ktoner pool
        Kton::transfer_to_system(Origin::signed(101), 12000);
        // reward_per_share = 3600 / 36 = 100
        assert_eq!(Kton::reward_per_share(), 100);
        // now acc 11 can withdraw 3600 ring
        let free_balance = Ring::free_balance(&11);
        let ring_total_issuance = Ring::total_issuance();

        Contract::operate_with_contact(Origin::signed(11), 1000, 100);
        assert_eq!(Kton::reward_can_withdraw(&11), 3500);
        // acc 11's ring balance untouched
        assert_eq!(Ring::free_balance(&11), free_balance);
        assert_eq!(Ring::total_issuance(), ring_total_issuance - 100);

    });
}

#[test]
fn check_paying_gas() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(1).build(), || {
        set_reward_per_share_hundred(11);

        let free_balance = Ring::free_balance(&11);
        let ring_total_issuance = Ring::total_issuance();

        let can_withdraw_amount = Kton::reward_can_withdraw(&11);
        // acc 11 can only withdraw 3600 ring
        assert_eq!(can_withdraw_amount, 3600);
        // acc 11 spent 4000 ring buying gas and no gas left
        Contract::operate_with_contact(Origin::signed(11), 4000, 4000);
        // buying gas should first deduct ring from reward
        // so it would consume all 3600 ring
        assert_eq!(Kton::reward_can_withdraw(&11), 0);
        // then buying gas should deduct ring on acc's free balance
        assert_eq!(Ring::free_balance(&11), free_balance - 400);
        // paying for gas costs 4000 ring in total
        // so the total_issuance would be 4000 less
        assert_eq!(Ring::total_issuance(), ring_total_issuance - 4000);

    });
}