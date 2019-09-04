// --- external ---
use runtime_io::with_externalities;
use srml_support::{
    assert_err,
    traits::{Currency, LockIdentifier, LockableCurrency, WithdrawReasons},
};
// --- custom ---
use super::{ExtBuilder, Kton, Origin};

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
