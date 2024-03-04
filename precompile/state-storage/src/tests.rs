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
	ExtBuilder, PCall, PrecompilesValue, Runtime, System, TestPrecompiles,
};
// moonbeam
use precompile_utils::{
	prelude::{RuntimeHelper, UnboundedBytes},
	testing::{PrecompileTesterExt, PrecompilesModifierTester},
};
// substrate
use frame_support::{StorageHasher, Twox128};

fn precompiles() -> TestPrecompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn selectors() {
	assert!(PCall::state_storage_at_selectors().contains(&0x78943fb7));
}

#[test]
fn modifier() {
	ExtBuilder::default().build().execute_with(|| {
		let mut tester = PrecompilesModifierTester::new(PrecompilesValue::get(), Alice, Precompile);
		tester.test_view_modifier(PCall::state_storage_at_selectors());
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile, vec![1u8, 2u8, 3u8, 4u8])
			.execute_reverts(|output| output == b"Unknown selector");
	});
}

#[test]
fn test_state_storage() {
	ExtBuilder::default().with_balances(vec![(Alice.into(), 100)]).build().execute_with(|| {
		System::set_block_number(5);

		let mut key = vec![0u8; 32];
		key[0..16].copy_from_slice(&Twox128::hash(b"System"));
		key[16..32].copy_from_slice(&Twox128::hash(b"Number"));

		precompiles()
			.prepare_test(Alice, Precompile, PCall::state_storage_at { key: key.into() })
			.expect_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())
			.expect_no_logs()
			.execute_returns(UnboundedBytes::from(&5u64.to_le_bytes()));
	});
}

#[test]
fn test_storage_filter() {
	ExtBuilder::default().with_balances(vec![(Alice.into(), 100)]).build().execute_with(|| {
		let mut key = vec![0u8; 32];
		key[0..16].copy_from_slice(&Twox128::hash(b"EVM"));
		key[16..32].copy_from_slice(&Twox128::hash(b"AccountCodes"));

		precompiles()
			.prepare_test(Alice, Precompile, PCall::state_storage_at { key: key.into() })
			.expect_no_logs()
			.execute_reverts(|output| output == b"Read restriction");
	});
}
