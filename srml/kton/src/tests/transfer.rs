// --- std ---
use std::u64::MAX as MAX_U64;
// --- external ---
use rand::{thread_rng, Rng};
use runtime_io::with_externalities;
use srml_support::{assert_err, assert_ok, traits::Currency};
// --- custom ---
use super::{uniform_range, ExtBuilder, Kton, Origin};

const ROUND: usize = 100000;

#[test]
fn regular_transfer() {
    accounts![_; Alice(644), Bob(755)];

    let mut rng = thread_rng();

    for value in uniform_range(1, MAX_U64, ROUND) {
        with_externalities(
            &mut ExtBuilder::default().existential_deposit(0).build(),
            || {
                Kton::deposit_creating(&Alice, value);

                let trans_value = rng.gen_range(0, value);

                assert_ok!(Kton::transfer(Origin::signed(Alice), Bob, trans_value));
                assert_eq!(Kton::free_balance(&Alice), value - trans_value);
                assert_eq!(Kton::free_balance(&Bob), trans_value);
            },
        );
    }
}

#[test]
fn underflow_transfer() {
    accounts![_; Alice(644), Bob(755)];

    for value in uniform_range(1, MAX_U64, ROUND) {
        with_externalities(
            &mut ExtBuilder::default().existential_deposit(0).build(),
            || {
                Kton::deposit_creating(&Alice, value);

                assert_err!(
                    Kton::transfer(Origin::signed(Alice), Bob, MAX_U64),
                    "balance too low to send value"
                );
                assert_eq!(Kton::free_balance(&Alice), value);
                assert_eq!(Kton::free_balance(&Bob), 0);
            },
        );
    }
}

#[test]
fn overflow_transfer() {
    accounts![_; Alice(644), Bob(755)];

    for value in uniform_range(2, MAX_U64, ROUND) {
        with_externalities(
            &mut ExtBuilder::default().existential_deposit(0).build(),
            || {
                Kton::deposit_creating(&Alice, value);
                Kton::deposit_creating(&Bob, MAX_U64);

                assert_err!(
                    Kton::transfer(Origin::signed(Alice), Bob, value - 1),
                    "destination balance too high to receive value"
                );
                assert_eq!(Kton::free_balance(&Alice), value);
                assert_eq!(Kton::free_balance(&Bob), MAX_U64);
            },
        );
    }
}

#[test]
fn edge_transfer() {
    accounts![_; Alice(644), Bob(755)];

    for value in uniform_range(1, MAX_U64, ROUND) {
        with_externalities(
            &mut ExtBuilder::default().existential_deposit(0).build(),
            || {
                Kton::deposit_creating(&Alice, value);

                assert_ok!(Kton::transfer(Origin::signed(Alice), Bob, value));
                assert_eq!(Kton::free_balance(&Alice), 0);
                assert_eq!(Kton::free_balance(&Bob), value);

                let diff = MAX_U64 - value;

                Kton::deposit_creating(&Alice, MAX_U64);

                assert_ok!(Kton::transfer(Origin::signed(Alice), Bob, diff));
                assert_eq!(Kton::free_balance(&Alice), value);
                assert_eq!(Kton::free_balance(&Bob), MAX_U64);
            },
        );
    }

    with_externalities(
        &mut ExtBuilder::default().existential_deposit(0).build(),
        || {
            Kton::deposit_creating(&Alice, 0);

            assert_err!(
                Kton::transfer(Origin::signed(Alice), Bob, 1),
                "balance too low to send value"
            );

            Kton::deposit_creating(&Alice, MAX_U64);
            Kton::deposit_creating(&Bob, 2);

            assert_err!(
                Kton::transfer(Origin::signed(Bob), Alice, 1),
                "destination balance too high to receive value"
            );
        },
    );
}

#[test]
fn corner_transfer() {
    with_externalities(
        &mut ExtBuilder::default().existential_deposit(0).build(),
        || {
            let accounts = uniform_range(0, MAX_U64, ROUND);

            Kton::deposit_creating(&accounts[0], MAX_U64);

            for i in 0..ROUND - 1 {
                let (from, to) = (accounts[i], accounts[i + 1]);

                assert_ok!(Kton::transfer(Origin::signed(from), to, MAX_U64));
                assert_eq!(Kton::free_balance(&from), 0);
                assert_eq!(Kton::free_balance(&to), MAX_U64);
            }
        },
    );

    with_externalities(
        &mut ExtBuilder::default().existential_deposit(0).build(),
        || {
            accounts![_; Alice(644), Bob(755)];

            Kton::deposit_creating(&Alice, 0);
            Kton::deposit_creating(&Bob, MAX_U64);

            assert_err!(
                Kton::transfer(Origin::signed(Alice), Bob, 1),
                "balance too low to send value"
            );
        },
    );
}

#[test]
fn random_transfer() {
    with_externalities(
        &mut ExtBuilder::default().existential_deposit(0).build(),
        || {
            let round = ROUND.checked_mul(2).unwrap_or(MAX_U64 as _);
            let accounts = uniform_range(0, MAX_U64, round);

            for account_and_value in &accounts {
                Kton::deposit_creating(account_and_value, *account_and_value);
            }

            let mut rng = thread_rng();

            for from in &accounts {
                let _ = Kton::transfer(
                    Origin::signed(*from),
                    accounts[rng.gen_range(0, round)],
                    rng.gen_range(0, MAX_U64),
                );
            }
        },
    );
}
