use primitives::traits::OnInitialize;
use runtime_io::with_externalities;
use srml_support::{assert_eq_uvec, assert_err, assert_noop, assert_ok, EnumerableStorageMap};
use srml_support::traits::{Currency, ReservableCurrency, WithdrawReason, WithdrawReasons};
use mock::*;
use super::*;
use super::MONTH_IN_SECONDS;

#[test]
fn test_env_build() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(0).build(), || {
        check_exposure_all();

        assert_eq!(Staking::bonded(&11), Some(10));
        assert_eq!(Staking::ledger(&10), Some(StakingLedgers {
            stash: 11,
            total_power: (100 * COIN / 10000) as u128,
            active_power: (100 * COIN / 10000) as u128,
            total_ring: 100 * COIN,
            regular_ring: 100 * COIN,
            active_ring: 100 * COIN,
            total_kton: 0,
            active_kton: 0,
            regular_items: vec![RegularItem {value: 100 * COIN, expire_time: 12 * MONTH_IN_SECONDS as u64}],
            unlocking: vec![]
        }));

        assert_eq!(Kton::free_balance(&11), COIN / 100);
        assert_eq!(Kton::total_issuance(), 16 * COIN / 100);
    });
}

#[test]
fn normal_kton_should_work() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(0).build(), || {

        Kton::deposit_creating(&1001, 10 * COIN);
        assert_ok!(Staking::bond(Origin::signed(1001), 1000, StakingBalance::Kton(10 * COIN), RewardDestination::Stash, 0));
        assert_eq!(Staking::ledger(&1000), Some(StakingLedgers {
            stash: 1001,
            total_power: (10 * COIN) as u128,
            active_power: (10 * COIN) as u128,
            total_ring: 0,
            regular_ring: 0,
            active_ring: 0,
            total_kton: 10 * COIN,
            active_kton: 10 * COIN,
            regular_items: vec![],
            unlocking: vec![]
        }));

        assert_eq!(Kton::locks(&1001), vec![kton::BalanceLock {id: STAKING_ID, amount: 10 * COIN, until: u64::max_value(), reasons: WithdrawReasons::all()}]);

        // promise_month should not work for kton
        Kton::deposit_creating(&2001, 10 * COIN);
        assert_ok!(Staking::bond(Origin::signed(2001), 2000, StakingBalance::Kton(10 * COIN), RewardDestination::Stash, 12));
        assert_eq!(Staking::ledger(&2000),Some(StakingLedgers {
            stash: 2001,
            total_power: (10 * COIN) as u128,
            active_power: (10 * COIN) as u128,
            total_ring: 0,
            regular_ring: 0,
            active_ring: 0,
            total_kton: 10 * COIN,
            active_kton: 10 * COIN,
            regular_items: vec![],
            unlocking: vec![]
        }));

    });
}

#[test]
fn regular_unbond_and_withdraw_should_work() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(0).build(), || {

        Timestamp::set_timestamp(13 * MONTH_IN_SECONDS as u64);
        Ring::deposit_creating(&11, 1000 * COIN);
        Staking::unbond(Origin::signed(10), StakingBalance::Ring(10 * COIN));
        assert_eq!(Staking::ledger(&10), Some(StakingLedgers {
            stash: 11,
            total_power: (100 * COIN / 10000) as u128,
            active_power: (90 * COIN / 10000) as u128,
            total_ring: 100 * COIN,
            regular_ring: 90 * COIN,
            active_ring: 90 * COIN,
            total_kton: 0,
            active_kton: 0,
            regular_items: vec![RegularItem {value: 90 * COIN, expire_time: 12 * MONTH_IN_SECONDS as u64}],
            unlocking: vec![UnlockChunk { value: StakingBalance::Ring(10000000000), era: 3, dt_power: 1000000}]
        }));

        Staking::unbond(Origin::signed(10), StakingBalance::Ring(20 * COIN));
        assert_eq!(Staking::ledger(&10), Some(StakingLedgers {
            stash: 11,
            total_power: (100 * COIN / 10000) as u128,
            active_power: (70 * COIN / 10000) as u128,
            total_ring: 100 * COIN,
            regular_ring: 70 * COIN,
            active_ring: 70 * COIN,
            total_kton: 0,
            active_kton: 0,
            regular_items: vec![RegularItem {value: 70 * COIN, expire_time: 12 * MONTH_IN_SECONDS as u64 }],
            unlocking: vec![UnlockChunk { value: StakingBalance::Ring(10000000000), era: 3, dt_power: 1000000},
                            UnlockChunk { value: StakingBalance::Ring(20000000000), era: 3, dt_power: 2000000}]
        }));

        // more than active ring
        Staking::unbond(Origin::signed(10), StakingBalance::Ring(120 * COIN));
        assert_eq!(Staking::ledger(&10), Some(StakingLedgers {
            stash: 11,
            total_power: (100 * COIN / 10000) as u128,
            active_power: 0,
            total_ring: 100 * COIN,
            regular_ring: 0,
            active_ring: 0,
            total_kton: 0,
            active_kton: 0,
            regular_items: vec![], // should be cleared
            unlocking: vec![UnlockChunk { value: StakingBalance::Ring(10000000000), era: 3, dt_power: 1000000},
                            UnlockChunk { value: StakingBalance::Ring(20000000000), era: 3, dt_power: 2000000},
                            UnlockChunk { value: StakingBalance::Ring(70000000000), era: 3, dt_power: 7000000}
            ]
        }));

        start_era(3);

        assert_ok!(Staking::withdraw_unbonded(Origin::signed(10)));
        assert_eq!(Staking::ledger(&10), Some(StakingLedgers {
            stash: 11,
            total_power: 0,
            active_power: 0,
            total_ring: 0,
            regular_ring: 0,
            active_ring: 0,
            total_kton: 0,
            active_kton: 0,
            regular_items: vec![], // should be cleared
            unlocking: vec![]
        }));
        let free_balance = Ring::free_balance(&11);
        assert_eq!(Ring::locks(&11), vec![balances::BalanceLock {id: STAKING_ID, amount: 0, until: u64::max_value(), reasons: WithdrawReasons::all()}]);
        assert_ok!(Ring::ensure_can_withdraw(&11, free_balance, WithdrawReason::Transfer, 0));
    });
}

#[test]
fn normal_unbond_should_work() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(0).build(), || {

        // unbond normal ring
        Ring::deposit_creating(&11, 1000 * COIN);
        let kton_free_balance = Kton::free_balance(&11);
        let mut origin_ledger = Staking::ledger(&10).unwrap();
        Staking::bond_extra(Origin::signed(11), StakingBalance::Ring(200 * COIN), 12);
        assert_eq!(Kton::free_balance(&11), kton_free_balance + 200 * COIN / 10000);

        origin_ledger.regular_items.push(RegularItem {value: 200 * COIN, expire_time: 12 * MONTH_IN_SECONDS as u64});
        assert_eq!(Staking::ledger(&10), Some(StakingLedgers {
            stash: 11,
            total_power: origin_ledger.total_power + (200 * COIN / 10000) as u128,
            active_power: origin_ledger.active_power + (200 * COIN / 10000) as u128,
            total_ring: origin_ledger.total_ring + 200 * COIN,
            regular_ring: origin_ledger.regular_ring + 200 * COIN,
            active_ring: origin_ledger.active_ring + 200 * COIN,
            total_kton: origin_ledger.total_kton,
            active_kton: origin_ledger.active_kton,
            regular_items: origin_ledger.regular_items,
            unlocking: origin_ledger.unlocking
        }));

        assert_eq!(Kton::free_balance(&11), 300 * COIN / 10000);
        let mut origin_ledger = Staking::ledger(&10).unwrap();
        // actually acc 11 only has 0.03 Kton
        // we try to bond 1 kton
        assert_ok!(Staking::bond_extra(Origin::signed(11), StakingBalance::Kton(COIN), 0));
        assert_eq!(Staking::ledger(&10), Some(StakingLedgers {
            stash: 11,
            total_power: origin_ledger.total_power + (300 * COIN / 10000) as u128,
            active_power: origin_ledger.active_power + (300 * COIN / 10000) as u128,
            total_ring: origin_ledger.total_ring,
            regular_ring: origin_ledger.regular_ring,
            active_ring: origin_ledger.active_ring,
            total_kton: origin_ledger.total_kton + 300 * COIN / 10000,
            active_kton: origin_ledger.active_kton + 300 * COIN / 10000,
            regular_items: origin_ledger.regular_items,
            unlocking: origin_ledger.unlocking
        }));

        assert_ok!(Staking::unbond(Origin::signed(10), StakingBalance::Kton(300 * COIN / 10000)));

    });
}

#[test]
fn punished_unbond_should_work() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(0).build(), || {
        Ring::deposit_creating(&1001, 100 * COIN);
        Kton::deposit_creating(&1001, COIN / 100000);

        // timestamp now is 0.
        // free balance of kton is too low to work
        assert_ok!(Staking::bond(Origin::signed(1001), 1000, StakingBalance::Ring(10 * COIN), RewardDestination::Stash, 36));
        assert_eq!(Staking::ledger(&1000), Some(StakingLedgers {
            stash: 1001,
            total_power: (10 * COIN / 10000) as u128,
            active_power: (10 * COIN / 10000) as u128,
            total_ring: 10 * COIN,
            regular_ring: 10 * COIN,
            active_ring: 10 * COIN,
            total_kton: 0,
            active_kton: 0,
            regular_items: vec![RegularItem { value: 10 * COIN, expire_time: 36 * MONTH_IN_SECONDS as u64 }], // should be cleared
            unlocking: vec![]
        }));
        let origin_ledger = Staking::ledger(&1000).unwrap();
        let kton_free_balance = Kton::free_balance(&1001);
        assert_ok!(Staking::unbond_with_punish(Origin::signed(1000), 10 * COIN, MONTH_IN_SECONDS as u64 * 36));
        assert_eq!(Staking::ledger(&1000), Some(origin_ledger.clone()));
        assert_eq!(Kton::free_balance(&1001), kton_free_balance);


        // set more kton balance to make it work
        Kton::deposit_creating(&1001, 10 * COIN);
        let kton_free_balance = Kton::free_balance(&1001);
        assert_ok!(Staking::unbond_with_punish(Origin::signed(1000), 5 * COIN, MONTH_IN_SECONDS as u64 * 36));
        assert_eq!(Staking::ledger(&1000), Some(StakingLedgers {
            stash: 1001,
            total_power: origin_ledger.total_power,
            active_power: origin_ledger.active_power - (5 * COIN / 10000) as u128,
            total_ring: origin_ledger.total_ring,
            regular_ring: origin_ledger.regular_ring - 5 * COIN,
            active_ring: origin_ledger.active_ring - 5 * COIN,
            total_kton: origin_ledger.total_kton,
            active_kton: origin_ledger.active_kton,
            regular_items: vec![RegularItem { value: 5 * COIN, expire_time: 36 * MONTH_IN_SECONDS as u64 }],
            unlocking: vec![UnlockChunk { value: StakingBalance::Ring(5 * COIN), era: 3, dt_power: (5 * COIN / 10000) as u128 }]
        }));

        let kton_punishment = utils::compute_kton_return::<Test>(5 * COIN, 36);
        assert_eq!(Kton::free_balance(&1001), kton_free_balance - 3 * kton_punishment);

        // if regularItem.value == 0
        // the whole item should be be dropped
        assert_ok!(Staking::unbond_with_punish(Origin::signed(1000), 5 * COIN, MONTH_IN_SECONDS as u64 * 36));
        assert_eq!(Staking::ledger(&1000).unwrap().regular_items, vec![]);
    });
}


#[test]
fn transform_to_promomised_ring_should_work() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(0).build(), || {

        Ring::deposit_creating(&1001, 100 * COIN);
        assert_ok!(Staking::bond(Origin::signed(1001), 1000, StakingBalance::Ring(10 * COIN), RewardDestination::Stash, 0));
        let origin_ledger = Staking::ledger(&1000).unwrap();
        let kton_free_balance = Kton::free_balance(&1001);

        assert_ok!(Staking::promise_extra(Origin::signed(1000), 5 * COIN, 12));

        assert_eq!(Staking::ledger(&1000), Some(StakingLedgers {
            stash: 1001,
            total_power: origin_ledger.total_power,
            active_power: origin_ledger.active_power,
            total_ring: origin_ledger.total_ring,
            regular_ring: origin_ledger.regular_ring + 5 * COIN,
            active_ring: origin_ledger.active_ring,
            total_kton: origin_ledger.total_kton,
            active_kton: origin_ledger.active_kton,
            regular_items: vec![RegularItem { value: 5 * COIN, expire_time: 12 * MONTH_IN_SECONDS as u64 }],
            unlocking: vec![]
        }));

        assert_eq!(Kton::free_balance(&1001), kton_free_balance + (5 * COIN / 10000));

    });
}