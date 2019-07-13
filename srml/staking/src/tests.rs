use mock::*;
use phragmen;
use primitives::traits::OnInitialize;
use runtime_io::with_externalities;
use srml_support::{assert_eq_uvec, assert_noop, assert_ok, assert_err, EnumerableStorageMap};
use srml_support::traits::{Currency, ReservableCurrency};

use super::*;

#[inline]
fn build_basic_env() {
    // stash -> controller
    // 91 -> 81 (payee: stash)
    // 92 -> 82 (payee: controller)
    Ring::transfer(Origin::signed(100), 91, 1000_000 * COIN);
    Ring::transfer(Origin::signed(100), 92, 1000_000 * COIN);
    // for operation fee
    Ring::transfer(Origin::signed(100), 81, 10 * COIN);
    Ring::transfer(Origin::signed(100), 82, 10 * COIN);

    // acc 91 and 92 deposit kton
    Kton::deposit(Origin::signed(91), 100_000 * COIN, 36);
    Kton::deposit(Origin::signed(92), 100_000 * COIN, 36);

    // now acc 91 and 92 has about 36 kton
    Staking::bond(Origin::signed(91), 81, 20 * COIN, RewardDestination::Stash);
    Staking::bond(Origin::signed(92), 82, 20 * COIN, RewardDestination::Controller);

    assert_eq!(Staking::bonded(&91), Some(81));
    assert_eq!(Staking::bonded(&92), Some(82));

    assert_eq!(Staking::ledger(&81), Some(StakingLedger { stash: 91, total: 20 * COIN, active: 20 * COIN, unlocking: vec![] }));
    assert_eq!(Staking::ledger(&82), Some(StakingLedger { stash: 92, total: 20 * COIN, active: 20 * COIN, unlocking: vec![] }));

    // users can not use `bond` twice
    assert_err!(Staking::bond(Origin::signed(91), 92, 1 * COIN, RewardDestination::Stash), "stash already bonded");
    // acc 103 has not bonded yet
    assert_eq!(Staking::ledger(&103), None);
    Staking::bond_extra(Origin::signed(91), 1 * COIN);
    assert_eq!(Staking::ledger(&81), Some(StakingLedger { stash: 91, total: 21 * COIN, active: 21 * COIN, unlocking: vec![] }));



}

#[test]
fn test_env_build() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(1).build(), || {
        build_basic_env();

        check_exposure_all();
    });
}

