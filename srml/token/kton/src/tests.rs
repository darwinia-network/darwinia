#![cfg(test)]

use super::*;
use sr_io::with_externalities;
use srml_support::{assert_ok, assert_noop, assert_eq_uvec};
use mock::{Ring, System, Timestamp, Kton, Test, ExtBuilder, Origin};

#[test]
fn check_sys_account() {

    with_externalities(&mut ExtBuilder::default()
        .build(),
        || {
            let sys_account = Kton::sys_account();
            assert_eq!(sys_account, 42_u64);
        });


}