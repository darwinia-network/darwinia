extern crate evo_kton as kton;
extern crate rand;

#[macro_use]
mod support;

// --- std ---
use std::u64::MAX as MAX_U64;
// --- external ---
use rand::{thread_rng, Rng};
use runtime_io::with_externalities;
use srml_support::{
    assert_err, assert_ok,
    traits::{Currency, LockIdentifier, LockableCurrency, WithdrawReasons},
};
// --- custom ---
use support::{uniform_range, ExtBuilder, Kton, Origin};

const ROUND: usize = 100000;

#[test]
fn regular_lock() {
    accounts![_; Alice(644), Bob(755)];

    with_externalities(
        &mut ExtBuilder::default().existential_deposit(0).build(),
        || {
            let lock_id: LockIdentifier = [0; 8];

            Kton::deposit_creating(&Bob, 100);
            Kton::set_lock(lock_id, &Bob, 90, u64::max_value(), WithdrawReasons::all());

            assert_err!(
                Kton::transfer(Origin::signed(Bob), Alice, 20),
                "account liquidity restrictions prevent withdrawal"
            );
        },
    );
}

#[test]
fn underflow_lock() {}

#[test]
fn overflow_lock() {}

#[test]
fn edge_lock() {}

#[test]
fn corner_lock() {}

#[test]
fn random_lock() {}
