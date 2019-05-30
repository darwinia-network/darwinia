#![cfg(test)]

use super::*;
use runtime_io::with_externalities;
use srml_support::{assert_ok, assert_noop, assert_err};
use mock::{Ring, System, Timestamp, Test, ExtBuilder, Origin};
use srml_support::traits::{Currency, LockIdentifier, WithdrawReason, WithdrawReasons};

const ID_1: LockIdentifier = *b"1       ";

#[test]
fn basic_locking_should_work() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(0).build(),  || {
        assert_eq!(Ring::free_balance(&1), 10);
        Ring::set_lock(ID_1, &1, 9, u64::max_value(), WithdrawReasons::all());
        assert_noop!(<Ring as Currency<_>>::transfer(&1, &2, 5), "account liquidity restrictions prevent withdrawal");

    });
}