use primitives::traits::OnInitialize;
use runtime_io::with_externalities;
use srml_support::{assert_eq_uvec, assert_err, assert_noop, assert_ok, EnumerableStorageMap};
use srml_support::traits::{Currency, ReservableCurrency, WithdrawReason, WithdrawReasons};

use mock::*;
use phragmen;

use super::*;

#[inline]
fn build_basic_env() {
    // stash -> controller
    // 91 -> 90 (payee: stash)
    // 81 -> 80 (payee: controller)
    Ring::transfer(Origin::signed(100), 91, 1000_000 * COIN);
    Ring::transfer(Origin::signed(100), 81, 1000_000 * COIN);
    // for operation fee
    Ring::transfer(Origin::signed(100), 90, 10 * COIN);
    Ring::transfer(Origin::signed(100), 80, 10 * COIN);

    // acc 91 and 81 deposit kton
    Kton::deposit(Origin::signed(91), 100_000 * COIN, 36);
    Kton::deposit(Origin::signed(81), 100_000 * COIN, 36);

    // now acc 91 and 81 has about 36 kton
    Staking::bond(Origin::signed(91), 90, 20 * COIN, RewardDestination::Stash);
    Staking::bond(Origin::signed(81), 80, 20 * COIN, RewardDestination::Controller);

    assert_eq!(Staking::bonded(&91), Some(90));
    assert_eq!(Staking::bonded(&81), Some(80));

    assert_eq!(Staking::ledger(&90), Some(StakingLedger { stash: 91, total: 20 * COIN, active: 20 * COIN, unlocking: vec![] }));
    assert_eq!(Staking::ledger(&80), Some(StakingLedger { stash: 81, total: 20 * COIN, active: 20 * COIN, unlocking: vec![] }));

    // users can not use `bond` twice
    assert_err!(Staking::bond(Origin::signed(91), 90, 1 * COIN, RewardDestination::Stash), "stash already bonded");
    // acc 103 has not bonded yet
    assert_eq!(Staking::ledger(&103), None);
    Staking::bond_extra(Origin::signed(91), 1 * COIN);
    assert_eq!(Staking::ledger(&90), Some(StakingLedger { stash: 91, total: 21 * COIN, active: 21 * COIN, unlocking: vec![] }));

    assert_ok!(Staking::validate(Origin::signed(90), ValidatorPrefs::default()));
    assert_ok!(Staking::validate(Origin::signed(80), ValidatorPrefs::default()));
}

#[test]
fn test_env_build() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(1).build(), || {
        check_exposure_all();

        // initial build storage should work
        // controller in session.validators
        assert_eq!(Session::validators(), vec![10, 20]);
        // 21 - the minimum bonded
        assert_eq!(Staking::stakers(&21), Exposure { total: 1000, own: 1000, others: vec![IndividualExposure {who: 101, value: 0}]});
        assert_eq!(Staking::stakers(&11), Exposure { total: 100 * COIN, own: 100 * COIN, others: vec![]});
        // stash in staking.current_elected
        assert_eq!(Staking::current_elected(), vec![11, 21]);

        build_basic_env();

        start_era(1);
        assert_eq!(Session::validators(), vec![10, 90, 80]);
        assert_eq!(Staking::current_elected(), vec![11, 91, 81]);
    });
}

#[test]
fn offline_should_slash_and_disable() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(1).build(), || {

        build_basic_env();
        start_era(1);
        // make sure acc 91 has bonded all his kton
        let _ = Kton::make_free_balance_be(&91, 21 * COIN);
        assert_err!(Kton::ensure_can_withdraw(&91, 1, WithdrawReason::Transfer, 0), "account liquidity restrictions prevent withdrawal");

        assert_eq!(Staking::current_elected(), vec![11, 91, 81]);

        assert_eq!(Staking::validators(&91).unstake_threshold, 3);
        assert_eq!(Staking::offline_slash_grace(), 0);

        assert!(<Validators<Test>>::exists(&91));
        assert!(!is_disabled(90));
        // limit offline_count for acc 91 is 3
        // offline count = limit + 1
        Staking::on_offline_validator(90, 11);
        assert_eq!(Staking::slash_count(&91), 11);

        assert!(is_disabled(90));

        start_era(2);

        // acc 21-20 will not be a validator because it failed to meet the standard
        assert_eq!(Staking::current_elected(), vec![11, 81, 21]);
        assert!(!<Stakers<Test>>::exists(&91));
        // out of validator set, status related will be cleared
        assert_eq!(Session::validators(), vec![10, 80, 20]);
    });
}

