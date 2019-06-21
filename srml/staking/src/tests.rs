// Copyright 2017-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Tests for the module.

#![cfg(test)]

use super::*;
use runtime_io::with_externalities;
use phragmen;
use primitives::PerU128;
use srml_support::{assert_ok, assert_noop, assert_eq_uvec, EnumerableStorageMap};
use mock::{Ring, Kton, Session, Staking, System, Timestamp, Test, ExtBuilder, Origin};
use srml_support::traits::{Currency, ReservableCurrency, WithdrawReason, WithdrawReasons, LockIdentifier};
use kton::{ BalanceLock, DECIMALS};


#[inline]
fn construct_staking_env() {
	Kton::deposit(Origin::signed(101), 1000000, 12);
	Kton::deposit(Origin::signed(102), 1000000, 12);
	Kton::deposit(Origin::signed(103), 1000000, 12);

	Staking::bond(Origin::signed(101), 1, 100, RewardDestination::Stash);
	Staking::bond(Origin::signed(102), 2, 100, RewardDestination::Stash);
	Staking::bond(Origin::signed(103), 3, 100, RewardDestination::Controller);

	assert_eq!(Kton::total_issuance(), 300);
	assert_eq!(Ring::total_issuance(), 10007734000000000);

}

#[test]
fn basic_work() {
	with_externalities(&mut ExtBuilder::default()
		.existential_deposit(0).build(), || {
		construct_staking_env();
		assert_eq!(Kton::free_balance(&101), 100);
		assert_eq!(Staking::bonded(&101), Some(1));
		assert_eq!(Staking::ledger(&1), Some( StakingLedger {stash: 101, total: 100, active: 100, unlocking: vec![]}));
		assert_eq!(Kton::locks(&101), vec![ BalanceLock{id: STAKING_ID, amount: 100, until: u64::max_value(), reasons: WithdrawReasons::all()}]);
		Kton::deposit(Origin::signed(101), 1000000, 12);
		assert_ok!(Staking::bond_extra(Origin::signed(101), 100));
		assert_eq!(Staking::ledger(&1), Some( StakingLedger {stash: 101, total: 200, active: 200, unlocking: vec![]}));
		assert_eq!(Kton::locks(&101), vec![ BalanceLock{ id: STAKING_ID, amount: 200, until: u64::max_value(), reasons: WithdrawReasons::all()}]);

	});
}

#[test]
fn test_validate() {
	with_externalities(&mut ExtBuilder::default()
		.existential_deposit(0).build(), || {
		construct_staking_env();

		assert_eq!(<Validators<Test>>::enumerate().collect::<Vec<_>>(), vec![]);
		assert_eq!(Staking::slot_stake(), 0);
		assert_eq!(System::block_number(), 1);
		Staking::validate(Origin::signed(1), ValidatorPrefs::default());
		Staking::validate(Origin::signed(2), ValidatorPrefs::default());
		assert_eq!(Staking::validator_count(), 2);

		assert_eq!(<Validators<Test>>::enumerate().collect::<Vec<_>>(), vec![
			(102, ValidatorPrefs::default()),
			(101, ValidatorPrefs::default()),
		]);

		// Initial Era and session
		assert_eq!(Staking::current_era(), 0);
		assert_eq!(Session::current_index(), 0);

		// initial rewards
		assert_eq!(Staking::current_session_reward(), 10);

		assert_eq!(Staking::select_validators(), 100);
		assert_eq!(Staking::slot_stake(), 100);

	});
}

#[test]
fn should_not_work_without_rotate() {
	with_externalities(&mut ExtBuilder::default()
		.existential_deposit(0).build(), || {

		construct_staking_env();
		// Initial
		System::set_block_number(1);
		assert_eq!(Session::current_index(), 0);
		assert_eq!(Session::last_length_change(), 0);
		assert_eq!(Session::current_start(), 0);

		// block 3
		System::set_block_number(3);
		assert_ne!(Session::current_index(), 1);
		assert_eq!(Staking::current_era(), 0);

		System::set_block_number(6);
		assert_ne!(Session::current_index(), 2);
		assert_ne!(Staking::current_era(), 1);

	});
}

#[test]
fn session_era_epoch_should_work_well() {
	with_externalities(&mut ExtBuilder::default()
		.existential_deposit(0).build(), || {

		construct_staking_env();
		// Initial
		System::set_block_number(1);
		assert_eq!(System::block_number(), 1);
		assert_eq!(Session::current_index(), 0);
		assert_eq!(Session::last_length_change(), 0);
		assert_eq!(Session::current_start(), 0);
		// set in mock
		assert_eq!(Session::validators(), vec![10, 20]);

		// block 3
		System::set_block_number(3);
		Session::check_rotate_session(System::block_number());
		assert_eq!(Session::current_index(), 1);
		assert_eq!(Staking::current_era(), 0);
		Staking::validate(Origin::signed(1), ValidatorPrefs{ unstake_threshold: 3, validator_payment: 100});
		Staking::validate(Origin::signed(2), ValidatorPrefs {unstake_threshold: 3, validator_payment: 100});

		// block 6
		System::set_block_number(6);
		Session::check_rotate_session(System::block_number());
		assert_eq!(Session::current_index(), 2);
		assert_eq!(Staking::current_era(), 1);
		assert_eq!(Session::validators(), vec![2, 1]);
		assert_eq!(Staking::current_elected(), vec![102, 101]);
		assert_eq!(Staking::current_epoch(), 0);

		System::set_block_number(9);
		Session::check_rotate_session(System::block_number());
		assert_eq!(Session::current_index(), 3);
		assert_eq!(Staking::current_era(), 1);


		System::set_block_number(12);
		assert_eq!(Staking::current_epoch(), 0);
		Session::check_rotate_session(System::block_number());
		assert_eq!(Session::current_index(), 4);
		assert_eq!(Staking::current_epoch(), 1);
		// ring total issuance /( 5 * 2)
		assert_eq!(Staking::ideal_era_reward(), 998999226600000000);
	});
}




