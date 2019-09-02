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
            total_ring: 100 * COIN,
            total_deposit_ring: 100 * COIN,
            active_deposit_ring: 100 * COIN,
            active_ring: 100 * COIN,
            total_kton: 0,
            active_kton: 0,
            deposit_items: vec![TimeDepositItem {value: 100 * COIN, start_time: 0, expire_time: 12 * MONTH_IN_SECONDS as u64}],
            unlocking: vec![]
        }));

        assert_eq!(Kton::free_balance(&11), COIN / 100);
        assert_eq!(Kton::total_issuance(), 16 * COIN / 100);

        let origin_ledger = Staking::ledger(&10).unwrap();
        Ring::deposit_creating(&11, 100 * COIN);
        assert_ok!(Staking::bond_extra(Origin::signed(11), StakingBalance::Ring(20 * COIN), 13));
        assert_eq!(Staking::ledger(&10), Some(StakingLedgers {
            stash: 11,
            total_ring: origin_ledger.total_ring + 20 * COIN,
            total_deposit_ring: origin_ledger.total_deposit_ring + 20 * COIN,
            active_deposit_ring: origin_ledger.active_deposit_ring + 20 * COIN,
            active_ring: origin_ledger.active_ring + 20 * COIN,
            total_kton: 0,
            active_kton: 0,
            deposit_items: vec![TimeDepositItem {value: 100 * COIN, start_time: 0, expire_time: 12 * MONTH_IN_SECONDS as u64},
                                TimeDepositItem {value: 20 * COIN, start_time: 0, expire_time: 13 * MONTH_IN_SECONDS as u64}],
            unlocking: vec![]
        }));
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
            total_ring: 0,
            total_deposit_ring: 0,
            active_deposit_ring: 0,
            active_ring: 0,
            total_kton: 10 * COIN,
            active_kton: 10 * COIN,
            deposit_items: vec![],
            unlocking: vec![]
        }));

        assert_eq!(Kton::locks(&1001), vec![kton::BalanceLock {id: STAKING_ID, amount: 10 * COIN, until: u64::max_value(), reasons: WithdrawReasons::all()}]);

        // promise_month should not work for kton
        Kton::deposit_creating(&2001, 10 * COIN);
        assert_ok!(Staking::bond(Origin::signed(2001), 2000, StakingBalance::Kton(10 * COIN), RewardDestination::Stash, 12));
        assert_eq!(Staking::ledger(&2000),Some(StakingLedgers {
            stash: 2001,
            total_ring: 0,
            total_deposit_ring: 0,
            active_deposit_ring: 0,
            active_ring: 0,
            total_kton: 10 * COIN,
            active_kton: 10 * COIN,
            deposit_items: vec![],
            unlocking: vec![]
        }));

    });
}

#[test]
fn time_deposit_ring_unbond_and_withdraw_should_work() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(0).build(), || {

        Timestamp::set_timestamp(13 * MONTH_IN_SECONDS as u64);
        Ring::deposit_creating(&11, 1000 * COIN);
        Staking::unbond(Origin::signed(10), StakingBalance::Ring(10 * COIN));
        assert_eq!(Staking::ledger(&10), Some(StakingLedgers {
            stash: 11,
            total_ring: 100 * COIN,
            total_deposit_ring: 100 * COIN,
            active_deposit_ring: 90 * COIN,
            active_ring: 90 * COIN,
            total_kton: 0,
            active_kton: 0,
            deposit_items: vec![TimeDepositItem {value: 90 * COIN, start_time: 0, expire_time: 12 * MONTH_IN_SECONDS as u64}],
            unlocking: vec![UnlockChunk { value: StakingBalance::Ring(10000000000), era: 3, is_time_deposit: true }]
        }));

        Staking::unbond(Origin::signed(10), StakingBalance::Ring(20 * COIN));
        assert_eq!(Staking::ledger(&10), Some(StakingLedgers {
            stash: 11,
            total_ring: 100 * COIN,
            total_deposit_ring: 100 * COIN,
            active_deposit_ring: 70 * COIN,
            active_ring: 70 * COIN,
            total_kton: 0,
            active_kton: 0,
            deposit_items: vec![TimeDepositItem {value: 70 * COIN, start_time: 0, expire_time: 12 * MONTH_IN_SECONDS as u64 }],
            unlocking: vec![UnlockChunk { value: StakingBalance::Ring(10000000000), era: 3, is_time_deposit: true},
                            UnlockChunk { value: StakingBalance::Ring(20000000000), era: 3, is_time_deposit: true}]
        }));

        // more than active ring
        Staking::unbond(Origin::signed(10), StakingBalance::Ring(120 * COIN));
        assert_eq!(Staking::ledger(&10), Some(StakingLedgers {
            stash: 11,
            total_ring: 100 * COIN,
            total_deposit_ring: 100 * COIN,
            active_deposit_ring: 0,
            active_ring: 0,
            total_kton: 0,
            active_kton: 0,
            deposit_items: vec![], // should be cleared
            unlocking: vec![UnlockChunk { value: StakingBalance::Ring(10000000000), era: 3, is_time_deposit: true},
                            UnlockChunk { value: StakingBalance::Ring(20000000000), era: 3, is_time_deposit: true},
                            UnlockChunk { value: StakingBalance::Ring(70000000000), era: 3, is_time_deposit: true},
            ]
        }));

        start_era(3);

        assert_ok!(Staking::withdraw_unbonded(Origin::signed(10)));
        assert_eq!(Staking::ledger(&10), Some(StakingLedgers {
            stash: 11,
            total_ring: 0,
            total_deposit_ring: 0,
            active_deposit_ring: 0,
            active_ring: 0,
            total_kton: 0,
            active_kton: 0,
            deposit_items: vec![], // should be cleared
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

        origin_ledger.deposit_items.push(TimeDepositItem {value: 200 * COIN, start_time: 0, expire_time: 12 * MONTH_IN_SECONDS as u64});
        assert_eq!(Staking::ledger(&10), Some(StakingLedgers {
            stash: 11,
            total_ring: origin_ledger.total_ring + 200 * COIN,
            total_deposit_ring: origin_ledger.total_deposit_ring + 200 * COIN,
            active_deposit_ring: origin_ledger.active_deposit_ring + 200 * COIN,
            active_ring: origin_ledger.active_ring + 200 * COIN,
            total_kton: origin_ledger.total_kton,
            active_kton: origin_ledger.active_kton,
            deposit_items: origin_ledger.deposit_items,
            unlocking: origin_ledger.unlocking
        }));

        assert_eq!(Kton::free_balance(&11), 300 * COIN / 10000);
        let mut origin_ledger = Staking::ledger(&10).unwrap();
        // actually acc 11 only has 0.03 Kton
        // we try to bond 1 kton
        assert_ok!(Staking::bond_extra(Origin::signed(11), StakingBalance::Kton(COIN), 0));
        assert_eq!(Staking::ledger(&10), Some(StakingLedgers {
            stash: 11,
            total_ring: origin_ledger.total_ring,
            total_deposit_ring: origin_ledger.total_deposit_ring,
            active_deposit_ring: origin_ledger.active_deposit_ring,
            active_ring: origin_ledger.active_ring,
            total_kton: origin_ledger.total_kton + 300 * COIN / 10000,
            active_kton: origin_ledger.active_kton + 300 * COIN / 10000,
            deposit_items: origin_ledger.deposit_items,
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
            total_ring: 10 * COIN,
            total_deposit_ring: 10 * COIN,
            active_deposit_ring: 10 * COIN,
            active_ring: 10 * COIN,
            total_kton: 0,
            active_kton: 0,
            deposit_items: vec![TimeDepositItem { value: 10 * COIN, start_time: 0, expire_time: 36 * MONTH_IN_SECONDS as u64 }], // should be cleared
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
            total_ring: origin_ledger.total_ring,
            total_deposit_ring: origin_ledger.total_deposit_ring,
            active_deposit_ring: origin_ledger.active_deposit_ring - 5 * COIN,
            active_ring: origin_ledger.active_ring - 5 * COIN,
            total_kton: origin_ledger.total_kton,
            active_kton: origin_ledger.active_kton,
            deposit_items: vec![TimeDepositItem { value: 5 * COIN, start_time: 0, expire_time: 36 * MONTH_IN_SECONDS as u64 }],
            unlocking: vec![UnlockChunk { value: StakingBalance::Ring(5 * COIN), era: 3, is_time_deposit: true }]
        }));

        let kton_punishment = utils::compute_kton_return::<Test>(5 * COIN, 36);
        assert_eq!(Kton::free_balance(&1001), kton_free_balance - 3 * kton_punishment);

        // if deposit_item.value == 0
        // the whole item should be be dropped
        assert_ok!(Staking::unbond_with_punish(Origin::signed(1000), 5 * COIN, MONTH_IN_SECONDS as u64 * 36));
        assert_eq!(Staking::ledger(&1000).unwrap().deposit_items, vec![]);
    });
}


#[test]
fn transform_to_promised_ring_should_work() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(0).build(), || {

        Ring::deposit_creating(&1001, 100 * COIN);
        assert_ok!(Staking::bond(Origin::signed(1001), 1000, StakingBalance::Ring(10 * COIN), RewardDestination::Stash, 0));
        let origin_ledger = Staking::ledger(&1000).unwrap();
        let kton_free_balance = Kton::free_balance(&1001);

        assert_ok!(Staking::promise_extra(Origin::signed(1000), 5 * COIN, 12));

        assert_eq!(Staking::ledger(&1000), Some(StakingLedgers {
            stash: 1001,
            total_ring: origin_ledger.total_ring,
            total_deposit_ring: origin_ledger.total_deposit_ring + 5 * COIN,
            active_deposit_ring: origin_ledger.active_deposit_ring + 5 * COIN,
            active_ring: origin_ledger.active_ring,
            total_kton: origin_ledger.total_kton,
            active_kton: origin_ledger.active_kton,
            deposit_items: vec![ TimeDepositItem { value: 5 * COIN, start_time: 0, expire_time: 12 * MONTH_IN_SECONDS as u64 }],
            unlocking: vec![]
        }));

        assert_eq!(Kton::free_balance(&1001), kton_free_balance + (5 * COIN / 10000));

    });
}

#[test]
fn expired_ring_should_capable_to_promise_again() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(0).build(), || {

        Ring::deposit_creating(&1001, 100 * COIN);
        assert_ok!(Staking::bond(Origin::signed(1001), 1000, StakingBalance::Ring(10 * COIN), RewardDestination::Stash, 12));
        let origin_ledger = Staking::ledger(&1000).unwrap();
        Timestamp::set_timestamp(13 * MONTH_IN_SECONDS as u64);
        assert_ok!(Staking::promise_extra(Origin::signed(1000), 5 * COIN, 13));
        assert_eq!(Staking::ledger(&1000), Some(StakingLedgers {
            stash: 1001,
            total_ring: origin_ledger.total_ring,
            total_deposit_ring: 5 * COIN,
            active_deposit_ring: 5 * COIN,
            active_ring: origin_ledger.active_ring,
            total_kton: origin_ledger.total_kton,
            active_kton: origin_ledger.active_kton,
            // old deposit_item with 12 months promised removed
            deposit_items: vec![ TimeDepositItem { value: 5 * COIN, start_time: 33696000, expire_time: 26 * MONTH_IN_SECONDS as u64 }],
            unlocking: vec![]
        }));
    });
}

#[test]
fn inflation_should_be_correct() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(0).build(), || {

        let initial_issuance = 1_200_000_000 * COIN;
        let surplus_needed = initial_issuance - Ring::total_issuance();
        Ring::deposit_into_existing(&11, surplus_needed);
        assert_eq!(Ring::total_issuance(), initial_issuance);

        start_era(11);
        let current_era_total_reward = Staking::current_era_total_reward();
        // ErasPerEpoch = 10
        assert_eq!(current_era_total_reward, 88000000 * COIN / 10);
    });
}

#[test]
fn reward_should_work_correctly() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(0).build(), || {
        // create controller account
        Ring::deposit_creating(&2000, COIN);
        Ring::deposit_creating(&1000, COIN);
        Ring::deposit_creating(&200, COIN);
        // new validator
        Ring::deposit_creating(&2001, 2000 * COIN);
        Kton::deposit_creating(&2001, 10 * COIN);
        // new validator
        Ring::deposit_creating(&1001,  300 * COIN);
        Kton::deposit_creating(&1001, 1 * COIN);
        // handle some dirty work
        Ring::deposit_creating(&201, 2000 * COIN);
        Kton::deposit_creating(&201, 10 * COIN);
        assert_eq!(Kton::free_balance(&201), 10 * COIN);

        // 2001-2000
        Staking::bond(Origin::signed(2001), 2000, StakingBalance::Ring(300 * COIN), RewardDestination::Controller, 12);
        Staking::bond_extra(Origin::signed(2001), StakingBalance::Kton(1 * COIN), 0);
        // 1001-1000
        Staking::bond(Origin::signed(1001), 1000, StakingBalance::Ring(300 * COIN), RewardDestination::Controller, 12);
        Staking::bond_extra(Origin::signed(1001), StakingBalance::Kton(1 * COIN), 0);
        let ring_pool = Staking::ring_pool();
        let kton_pool = Staking::kton_pool();
        // 201-200
        Staking::bond(Origin::signed(201), 200, StakingBalance::Ring(3000 * COIN - ring_pool), RewardDestination::Stash, 12);
        Staking::bond_extra(Origin::signed(201), StakingBalance::Kton(10 * COIN - kton_pool), 0);
        // ring_pool and kton_pool
        assert_eq!(Staking::ring_pool(), 3000 * COIN);
        assert_eq!(Staking::kton_pool(), 10 * COIN);
        // 1/5 ring_ppol and 1/5 kton_pool
        Staking::validate(Origin::signed(2000), [0;8].to_vec(), 0, 3);
        Staking::nominate(Origin::signed(1000), vec![2001]);

        assert_eq!(Staking::ledger(&2000).unwrap().active_kton, 1 * COIN);
        assert_eq!(Staking::ledger(&2000).unwrap().active_ring, 300 * COIN);
        assert_eq!(Staking::slashable_balance_of(&2001), 600 * COIN as u128);
        // 600COIN for rewarding ring bond-er
        // 600COIN for rewarding kton bond-er
        Staking::select_validators();
        Staking::reward_validator(&2001, 1200 * COIN);

        assert_eq!(Staking::stakers(2001),
                   Exposures {
                       total: 1200000000000,
                       own: 600000000000,
                       others: vec![IndividualExpo { who: 1001, value: 600000000000 }] });
        assert_eq!(Ring::free_balance(&2000), 601 * COIN);
        assert_eq!(Ring::free_balance(&1000), 601 * COIN);
    });
}

#[test]
fn slash_should_work() {
    with_externalities(&mut ExtBuilder::default()
        .existential_deposit(0).build(), || {

        Ring::deposit_creating(&1001, 100 * COIN);
        Kton::deposit_creating(&1001, 100 * COIN);

        Staking::bond(Origin::signed(1001), 1000, StakingBalance::Ring(50 * COIN), RewardDestination::Controller, 0);
        Staking::bond_extra(Origin::signed(1000), StakingBalance::Kton(50 * COIN), 0);
        Staking::validate(Origin::signed(1000), [0;8].to_vec(), 0, 3);
        // slash 1%
        Staking::slash_validator(&1001, 10_000_000);
        assert_eq!(Staking::ledger(&1000).unwrap().active_ring, 495 * COIN / 10);
        assert_eq!(Ring::free_balance(&1001), 995 * COIN / 10);
    });
}