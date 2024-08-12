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
use crate::mock::{
	Account::{Alice, Precompile},
	Deposit, *,
};
use darwinia_deposit::MILLISECS_PER_MONTH;
// moonbeam
use precompile_utils::testing::PrecompileTesterExt;
// polkadot-sdk
use sp_core::H160;

fn precompiles() -> TestPrecompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn selectors() {
	assert!(PCall::lock_selectors().contains(&0x998e4242));
	assert!(PCall::claim_selectors().contains(&0x4e71d92d));
	assert!(PCall::claim_with_penalty_selectors().contains(&0xfa04a9bf));
	assert!(PCall::migrate_selectors().contains(&0x8fd3ab80));
}

#[test]
fn lock_and_claim() {
	let alice: H160 = Alice.into();
	ExtBuilder::default().with_balances(vec![(alice, 300)]).build().execute_with(|| {
		// lock
		precompiles()
			.prepare_test(alice, Precompile, PCall::lock { amount: 200.into(), months: 1 })
			.execute_returns(true);
		assert!(Deposit::deposit_of(alice).is_some());

		// claim
		efflux(MILLISECS_PER_MONTH);
		precompiles().prepare_test(alice, Precompile, PCall::claim {}).execute_returns(true);
		assert!(Deposit::deposit_of(alice).is_none());
	});
}

#[test]
fn claim_with_penalty() {
	let alice: H160 = Alice.into();
	ExtBuilder::default().with_balances(vec![(alice, 300)]).build().execute_with(|| {
		// lock
		precompiles()
			.prepare_test(alice, Precompile, PCall::lock { amount: 200.into(), months: 1 })
			.execute_returns(true);
		assert!(Deposit::deposit_of(alice).is_some());

		// claim with penalty
		precompiles()
			.prepare_test(alice, Precompile, PCall::claim_with_penalty { id: 0 })
			.execute_returns(true);
		assert!(Deposit::deposit_of(alice).is_none());
	});
}
