use mock::*;
use phragmen;
use primitives::traits::OnInitialize;
use runtime_io::with_externalities;
use srml_support::{assert_eq_uvec, assert_noop, assert_ok, EnumerableStorageMap};
use srml_support::traits::{Currency, ReservableCurrency};

use super::*;

#[test]
fn basic_setup_works() {
    // Verifies initial conditions of mock
    with_externalities(&mut ExtBuilder::default()
        .build(), || {
        assert_eq!(Staking::bonded(&11), Some(10)); // Account 11 is stashed and locked, and account 10 is the controller
        assert_eq!(Staking::bonded(&21), Some(20)); // Account 21 is stashed and locked, and account 20 is the controller
        assert_eq!(Staking::bonded(&1), None);        // Account 1 is not a stashed

        // Account 10 controls the stash from account 11, which is 100 * balance_factor units
        assert_eq!(Staking::ledger(&10), Some(StakingLedger { stash: 11, total: 1000, active: 1000, unlocking: vec![] }));
        // Account 20 controls the stash from account 21, which is 200 * balance_factor units
        assert_eq!(Staking::ledger(&20), Some(StakingLedger { stash: 21, total: 1000, active: 1000, unlocking: vec![] }));
        // Account 1 does not control any stash
        assert_eq!(Staking::ledger(&1), None);

        // ValidatorPrefs are default, thus unstake_threshold is 3, other values are default for their type
        assert_eq!(<Validators<Test>>::enumerate().collect::<Vec<_>>(), vec![
            (31, ValidatorPrefs { unstake_threshold: 3, validator_payment: 0 }),
            (21, ValidatorPrefs { unstake_threshold: 3, validator_payment: 0 }),
            (11, ValidatorPrefs { unstake_threshold: 3, validator_payment: 0 })
        ]);

        // Account 100 is the default nominator
        assert_eq!(Staking::ledger(100), Some(StakingLedger { stash: 101, total: 500, active: 500, unlocking: vec![] }));
        assert_eq!(Staking::nominators(101), vec![11, 21]);

        // Account 10 is exposed by 1000 * balance_factor from their own stash in account 11 + the default nominator vote
        assert_eq!(Staking::stakers(11), Exposure { total: 1125, own: 1000, others: vec![IndividualExposure { who: 101, value: 125 }] });
        // Account 20 is exposed by 1000 * balance_factor from their own stash in account 21 + the default nominator vote
        assert_eq!(Staking::stakers(21), Exposure { total: 1375, own: 1000, others: vec![IndividualExposure { who: 101, value: 375 }] });

        // The number of validators required.
        assert_eq!(Staking::validator_count(), 2);

        // Initial Era and session
        assert_eq!(Staking::current_era(), 0);
        assert_eq!(Session::current_index(), 0);

        // initial rewards
        assert_eq!(Staking::current_session_reward(), 10);

        // initial slot_stake
        assert_eq!(Staking::slot_stake(), 1125); // Naive

        // initial slash_count of validators
        assert_eq!(Staking::slash_count(&11), 0);
        assert_eq!(Staking::slash_count(&21), 0);

        // All exposures must be correct.
        check_exposure_all();
    });
}
