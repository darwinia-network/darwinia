// This file is part of Darwinia.
//
// Copyright (C) Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

// darwinia
use crate::{
	mock::{Deposit, *},
	Deposit as DepositS, *,
};
// polkadot-sdk
use frame_support::{assert_ok, traits::OnIdle};

#[test]
fn migrate_should_work() {
	new_test_ext().execute_with(|| {
		let _ = Balances::deposit_creating(&account_id(), 2);

		<Deposits<Runtime>>::insert(
			1,
			BoundedVec::truncate_from(vec![
				DepositS { id: 0, value: 1, start_time: 0, expired_time: 0, in_use: false },
				DepositS { id: 0, value: 1, start_time: 0, expired_time: 0, in_use: true },
			]),
		);

		assert!(Deposit::deposit_of(1).is_some());
		assert_ok!(Deposit::migrate_for(RuntimeOrigin::signed(1), 1));
		assert!(Deposit::deposit_of(1).is_none());
	});
}

#[test]
fn on_idle_should_work() {
	fn mock_deposits(count: u16) -> BoundedVec<DepositS, ConstU32<512>> {
		BoundedVec::truncate_from(
			(0..count)
				.map(|id| DepositS { id, value: 1, start_time: 0, expired_time: 0, in_use: false })
				.collect(),
		)
	}

	new_test_ext().execute_with(|| {
		let _ = Balances::deposit_creating(&account_id(), 10_000);

		<Deposits<Runtime>>::insert(1, mock_deposits(512));
		<Deposits<Runtime>>::insert(2, mock_deposits(512));
		<Deposits<Runtime>>::insert(3, mock_deposits(512));
		assert_eq!(Deposit::deposit_of(1).unwrap().len(), 512);
		assert_eq!(Deposit::deposit_of(2).unwrap().len(), 512);
		assert_eq!(Deposit::deposit_of(3).unwrap().len(), 512);

		AllPalletsWithSystem::on_idle(0, Weight::zero());
		assert_eq!(Deposit::deposit_of(1).unwrap().len(), 512);
		assert_eq!(Deposit::deposit_of(2).unwrap().len(), 512);
		assert_eq!(Deposit::deposit_of(3).unwrap().len(), 512);
		AllPalletsWithSystem::on_idle(0, Weight::MAX);
		assert!(Deposit::deposit_of(1).is_none());
		assert!(Deposit::deposit_of(2).is_none());
		assert!(Deposit::deposit_of(3).is_none());

		// ---

		<Deposits<Runtime>>::insert(1, mock_deposits(512));
		<Deposits<Runtime>>::insert(2, mock_deposits(512));
		<Deposits<Runtime>>::insert(3, mock_deposits(512));
		assert_eq!(Deposit::deposit_of(1).unwrap().len(), 512);
		assert_eq!(Deposit::deposit_of(2).unwrap().len(), 512);
		assert_eq!(Deposit::deposit_of(3).unwrap().len(), 512);

		AllPalletsWithSystem::on_idle(0, Weight::zero().add_ref_time(256));
		assert_eq!(Deposit::deposit_of(1).unwrap().len(), 512);
		assert_eq!(Deposit::deposit_of(2).unwrap().len(), 512);
		// 512 - 256 / 10 * 10 = 252
		assert_eq!(Deposit::deposit_of(3).unwrap().len(), 262);
		AllPalletsWithSystem::on_idle(0, Weight::zero().add_ref_time(1_234));
		assert!(Deposit::deposit_of(1).is_none());
		// 512 => 520 weight
		// 1_234 - 520 * 2 = 194
		// 262 - 194 / 10 * 10 = 72
		assert_eq!(Deposit::deposit_of(2).unwrap().len(), 72);
		assert!(Deposit::deposit_of(3).is_none());
	});
}
