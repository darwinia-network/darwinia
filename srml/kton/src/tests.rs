
use runtime_io::with_externalities;
use srml_support::{assert_err, assert_noop, assert_ok};
use srml_support::traits::{Currency, ExistenceRequirement, Imbalance, WithdrawReason, WithdrawReasons, LockIdentifier};
use mock::{ExtBuilder, Kton, Origin, System, Test, Timestamp};
use super::*;

#[test]
fn transfer_should_work() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(0).build(), || {
        //
        Kton::deposit_creating(&1001, 100);
        assert_err!(Kton::transfer(Origin::signed(1001), 1000, 500), "balance too low to send value");
        assert_eq!(Kton::free_balance(&1000), 0);

    });
}

#[test]
fn lock_should_work() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(0).build(), || {

        let lock_id: LockIdentifier = *b"locklock";
        Kton::deposit_creating(&1001, 100);
        Kton::set_lock(lock_id, &1001, 90, u64::max_value(), WithdrawReasons::all());
        assert_err!(Kton::transfer(Origin::signed(1001), 1000, 20), "account liquidity restrictions prevent withdrawal");
    });
}

