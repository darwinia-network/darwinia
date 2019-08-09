use primitives::traits::OnInitialize;
use runtime_io::with_externalities;
use srml_support::{assert_eq_uvec, assert_err, assert_noop, assert_ok, EnumerableStorageMap};
use srml_support::traits::{Currency, ReservableCurrency, WithdrawReason, WithdrawReasons};

use mock::*;
use super::*;

// 3600 * 24 * 30
pub const MONTH: u64 = 3600 * 24 * 30;

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
            normal_ring: 0,
            regular_ring: 100 * COIN,
            active_ring: 100 * COIN,
            normal_kton: 0,
            active_kton: 0,
            regular_items: vec![RegularItem {value: 100 * COIN, expire_time: 12 * MONTH}],
            unlocking: vec![]
        }));
    });
}