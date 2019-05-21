#![cfg(test)]

use super::*;
use mock::{ExtBuilder, Kton, Origin, Ring, System, Test, Timestamp, Contract};
use runtime_io::with_externalities;
use srml_support::{assert_err, assert_noop, assert_ok};
use evo_support::traits::SystemCurrency;


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
        Contract::operate_with_contact(Origin::signed(11), 1000, 100);
        assert_eq!(Kton::reward_can_withdraw(&11), 2600);
        let new_free_balance = free_balance + 900;
        assert_eq!(Ring::free_balance(&11), new_free_balance);


    });
}
