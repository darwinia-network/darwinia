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

// crates.io
use sha3::{Digest, Keccak256};
// darwinia
use crate::{
	mock::{Account::*, *},
	*,
};
// moonbeam
use precompile_utils::{
	prelude::{Address, UnboundedBytes},
	testing::{PrecompileTesterExt, PrecompilesModifierTester},
};
// substrate
use frame_support::assert_ok;
use sp_core::{H256, U256};
use sp_std::str::from_utf8;

fn precompiles() -> TestPrecompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn selectors() {
	assert!(InternalCall::balance_of_selectors().contains(&0x70a08231));
	assert!(InternalCall::total_supply_selectors().contains(&0x18160ddd));
	assert!(InternalCall::approve_selectors().contains(&0x095ea7b3));
	assert!(InternalCall::allowance_selectors().contains(&0xdd62ed3e));
	assert!(InternalCall::transfer_selectors().contains(&0xa9059cbb));
	assert!(InternalCall::transfer_from_selectors().contains(&0x23b872dd));
	assert!(InternalCall::name_selectors().contains(&0x06fdde03));
	assert!(InternalCall::symbol_selectors().contains(&0x95d89b41));
	assert!(InternalCall::decimals_selectors().contains(&0x313ce567));

	assert!(InternalCall::mint_selectors().contains(&0x40c10f19));
	assert!(InternalCall::burn_selectors().contains(&0x9dc29fac));
	assert!(InternalCall::freeze_selectors().contains(&0x8d1fdf2f));
	assert!(InternalCall::thaw_selectors().contains(&0x5ea20216));
	assert!(InternalCall::transfer_ownership_selectors().contains(&0xf0350c04));

	assert_eq!(
		crate::SELECTOR_LOG_TRANSFER,
		&Keccak256::digest(b"Transfer(address,address,uint256)")[..]
	);

	assert_eq!(
		crate::SELECTOR_LOG_APPROVAL,
		&Keccak256::digest(b"Approval(address,address,uint256)")[..]
	);
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Assets::force_create(
			RuntimeOrigin::root(),
			TEST_ID.into(),
			Alice.into(),
			true,
			1
		));
		// This selector is only three bytes long when four are required.
		precompiles()
			.prepare_test(Alice, Precompile, vec![1u8, 2u8, 3u8])
			.execute_reverts(|output| output == b"Tried to read selector out of bounds");
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Assets::force_create(
			RuntimeOrigin::root(),
			TEST_ID.into(),
			Alice.into(),
			true,
			1
		));

		precompiles()
			.prepare_test(Alice, Precompile, vec![1u8, 2u8, 3u8, 4u8])
			.execute_reverts(|output| output == b"Unknown selector");
	});
}

#[test]
fn modifiers() {
	ExtBuilder::default().with_balances(vec![(Alice.into(), 1000)]).build().execute_with(|| {
		assert_ok!(Assets::force_create(
			RuntimeOrigin::root(),
			TEST_ID.into(),
			Alice.into(),
			true,
			1
		));
		let mut tester = PrecompilesModifierTester::new(precompiles(), Alice, Precompile);

		tester.test_view_modifier(InternalCall::balance_of_selectors());
		tester.test_view_modifier(InternalCall::total_supply_selectors());
		tester.test_default_modifier(InternalCall::approve_selectors());
		tester.test_view_modifier(InternalCall::allowance_selectors());
		tester.test_default_modifier(InternalCall::transfer_selectors());
		tester.test_default_modifier(InternalCall::transfer_from_selectors());
		tester.test_view_modifier(InternalCall::name_selectors());
		tester.test_view_modifier(InternalCall::symbol_selectors());
		tester.test_view_modifier(InternalCall::decimals_selectors());

		tester.test_default_modifier(InternalCall::mint_selectors());
		tester.test_default_modifier(InternalCall::burn_selectors());
		tester.test_default_modifier(InternalCall::freeze_selectors());
		tester.test_default_modifier(InternalCall::thaw_selectors());
		tester.test_default_modifier(InternalCall::transfer_ownership_selectors());
	});
}

#[test]
fn get_total_supply() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000), (Bob.into(), 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				TEST_ID.into(),
				Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(Alice.into()),
				TEST_ID.into(),
				Alice.into(),
				1000
			));

			precompiles()
				.prepare_test(Alice, Precompile, InternalCall::total_supply {})
				.expect_no_logs()
				.execute_returns(U256::from(1000u64));
		});
}

#[test]
fn get_balances_known_user() {
	ExtBuilder::default().with_balances(vec![(Alice.into(), 1000)]).build().execute_with(|| {
		assert_ok!(Assets::force_create(
			RuntimeOrigin::root(),
			TEST_ID.into(),
			Alice.into(),
			true,
			1
		));
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(Alice.into()),
			TEST_ID.into(),
			Alice.into(),
			1000
		));

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				InternalCall::balance_of { who: Address(Alice.into()) },
			)
			.expect_no_logs()
			.execute_returns(U256::from(1000u64));
	});
}

#[test]
fn get_balances_unknown_user() {
	ExtBuilder::default().with_balances(vec![(Alice.into(), 1000)]).build().execute_with(|| {
		assert_ok!(Assets::force_create(
			RuntimeOrigin::root(),
			TEST_ID.into(),
			Alice.into(),
			true,
			1
		));
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(Alice.into()),
			TEST_ID.into(),
			Alice.into(),
			1000
		));

		precompiles()
			.prepare_test(Alice, Precompile, InternalCall::balance_of { who: Address(Bob.into()) })
			.expect_no_logs()
			.execute_returns(U256::from(0u64));
	});
}

#[test]
fn approve() {
	ExtBuilder::default().with_balances(vec![(Alice.into(), 1000)]).build().execute_with(|| {
		assert_ok!(Assets::force_create(
			RuntimeOrigin::root(),
			TEST_ID.into(),
			Alice.into(),
			true,
			1
		));
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(Alice.into()),
			TEST_ID.into(),
			Alice.into(),
			1000
		));

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				InternalCall::approve { spender: Address(Bob.into()), value: 500.into() },
			)
			.expect_log(log3(
				Precompile,
				SELECTOR_LOG_APPROVAL,
				H256::from(Alice),
				H256::from(Bob),
				solidity::encode_event_data(U256::from(500)),
			))
			.execute_returns(true);
	});
}

#[test]
fn approve_saturating() {
	ExtBuilder::default().with_balances(vec![(Alice.into(), 1000)]).build().execute_with(|| {
		assert_ok!(Assets::force_create(
			RuntimeOrigin::root(),
			TEST_ID.into(),
			Alice.into(),
			true,
			1
		));
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(Alice.into()),
			TEST_ID.into(),
			Alice.into(),
			1000
		));

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				InternalCall::approve { spender: Address(Bob.into()), value: U256::MAX },
			)
			.execute_returns(true);
		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				InternalCall::allowance {
					owner: Address(Alice.into()),
					spender: Address(Bob.into()),
				},
			)
			.execute_returns(U256::from(u128::MAX));
	});
}

#[test]
fn check_allowance_existing() {
	ExtBuilder::default().with_balances(vec![(Alice.into(), 1000)]).build().execute_with(|| {
		assert_ok!(Assets::force_create(
			RuntimeOrigin::root(),
			TEST_ID.into(),
			Alice.into(),
			true,
			1
		));
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(Alice.into()),
			TEST_ID.into(),
			Alice.into(),
			1000
		));

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				InternalCall::approve { spender: Address(Bob.into()), value: 500.into() },
			)
			.execute_some();

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				InternalCall::allowance {
					owner: Address(Alice.into()),
					spender: Address(Bob.into()),
				},
			)
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns(U256::from(500u64));
	});
}

#[test]
fn check_allowance_not_existing() {
	ExtBuilder::default().with_balances(vec![(Alice.into(), 1000)]).build().execute_with(|| {
		assert_ok!(Assets::force_create(
			RuntimeOrigin::root(),
			TEST_ID.into(),
			Alice.into(),
			true,
			1
		));

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				InternalCall::allowance {
					owner: Address(Alice.into()),
					spender: Address(Bob.into()),
				},
			)
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns(U256::from(0u64));
	});
}

#[test]
fn transfer() {
	ExtBuilder::default().with_balances(vec![(Alice.into(), 1000)]).build().execute_with(|| {
		assert_ok!(Assets::force_create(
			RuntimeOrigin::root(),
			TEST_ID.into(),
			Alice.into(),
			true,
			1
		));
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(Alice.into()),
			TEST_ID.into(),
			Alice.into(),
			1000
		));

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				InternalCall::transfer { to: Address(Bob.into()), value: 400.into() },
			)
			.expect_log(log3(
				Precompile,
				SELECTOR_LOG_TRANSFER,
				H256::from(Alice),
				H256::from(Bob),
				solidity::encode_event_data(U256::from(400)),
			))
			.execute_returns(true);

		precompiles()
			.prepare_test(Bob, Precompile, InternalCall::balance_of { who: Address(Bob.into()) })
			.expect_no_logs()
			.execute_returns(U256::from(400));

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				InternalCall::balance_of { who: Address(Alice.into()) },
			)
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns(U256::from(600));
	});
}

#[test]
fn transfer_not_enough_founds() {
	ExtBuilder::default().with_balances(vec![(Alice.into(), 1000)]).build().execute_with(|| {
		assert_ok!(Assets::force_create(
			RuntimeOrigin::root(),
			TEST_ID.into(),
			Alice.into(),
			true,
			1
		));
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(Alice.into()),
			TEST_ID.into(),
			Alice.into(),
			1
		));

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				InternalCall::transfer { to: Address(Charlie.into()), value: 50.into() },
			)
			.execute_reverts(|output| {
				from_utf8(output).unwrap().contains("Dispatched call failed with error: ")
					&& from_utf8(output).unwrap().contains("BalanceLow")
			});
	});
}

#[test]
fn transfer_from() {
	ExtBuilder::default().with_balances(vec![(Alice.into(), 1000)]).build().execute_with(|| {
		assert_ok!(Assets::force_create(
			RuntimeOrigin::root(),
			TEST_ID.into(),
			Alice.into(),
			true,
			1
		));
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(Alice.into()),
			TEST_ID.into(),
			Alice.into(),
			1000
		));

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				InternalCall::approve { spender: Address(Bob.into()), value: 500.into() },
			)
			.execute_some();

		precompiles()
			.prepare_test(
				Bob, // Bob is the one sending transferFrom!
				Precompile,
				InternalCall::transfer_from {
					from: Address(Alice.into()),
					to: Address(Charlie.into()),
					value: 400.into(),
				},
			)
			.expect_log(log3(
				Precompile,
				SELECTOR_LOG_TRANSFER,
				H256::from(Alice),
				H256::from(Charlie),
				solidity::encode_event_data(U256::from(400)),
			))
			.execute_returns(true);

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				InternalCall::balance_of { who: Address(Alice.into()) },
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns(U256::from(600));

		precompiles()
			.prepare_test(Bob, Precompile, InternalCall::balance_of { who: Address(Bob.into()) })
			.expect_no_logs()
			.execute_returns(U256::from(0));

		precompiles()
			.prepare_test(
				Charlie,
				Precompile,
				InternalCall::balance_of { who: Address(Charlie.into()) },
			)
			.expect_no_logs()
			.execute_returns(U256::from(400));

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				InternalCall::allowance {
					owner: Address(Alice.into()),
					spender: Address(Bob.into()),
				},
			)
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns(U256::from(100u64));
	});
}

#[test]
fn transfer_from_non_incremental_approval() {
	ExtBuilder::default().with_balances(vec![(Alice.into(), 1000)]).build().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), TEST_ID.into(), Alice.into(), true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(Alice.into()), TEST_ID.into(), Alice.into(), 1000));

		// We first approve 500
		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				InternalCall::approve { spender: Address(Bob.into()), value: 500.into() },
			)
			.expect_log(log3(
				Precompile,
				SELECTOR_LOG_APPROVAL,
				H256::from(Alice),
				H256::from(Bob),
				solidity::encode_event_data(U256::from(500)),
			))
			.execute_returns(true);

		// We then approve 300. Non-incremental, so this is
		// the approved new value
		// Additionally, the gas used in this approval is higher because we
		// need to clear the previous one
		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				InternalCall::approve { spender: Address(Bob.into()), value: 300.into() },
			)
			.expect_log(log3(
				Precompile,
				SELECTOR_LOG_APPROVAL,
				H256::from(Alice),
				H256::from(Bob),
				solidity::encode_event_data(U256::from(300)),
			))
			.execute_returns(true);

		// This should fail, as now the new approved quantity is 300
		precompiles()
			.prepare_test(
				Bob, // Bob is the one sending transferFrom!
				Precompile,
				InternalCall::transfer_from {
					from: Address(Alice.into()),
					to: Address(Bob.into()),
					value: 500.into(),
				},
			)
			.execute_reverts(|output| {
				output == b"Dispatched call failed with error: Module(ModuleError { index: 4, error: [10, 0, 0, 0], \
					message: Some(\"Unapproved\") })"
			});
	});
}

#[test]
fn transfer_from_above_allowance() {
	ExtBuilder::default().with_balances(vec![(Alice.into(), 1000)]).build().execute_with(|| {
		assert_ok!(Assets::force_create(
			RuntimeOrigin::root(),
			TEST_ID.into(),
			Alice.into(),
			true,
			1
		));
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(Alice.into()),
			TEST_ID.into(),
			Alice.into(),
			1000
		));

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				InternalCall::approve { spender: Address(Bob.into()), value: 300.into() },
			)
			.execute_some();

		precompiles()
			.prepare_test(
				Bob, // Bob is the one sending transferFrom!
				Precompile,
				InternalCall::transfer_from {
					from: Address(Alice.into()),
					to: Address(Bob.into()),
					value: 400.into(),
				},
			)
			.execute_reverts(|output| {
				output
						== b"Dispatched call failed with error: Module(ModuleError { index: 4, error: [10, 0, 0, 0], \
					message: Some(\"Unapproved\") })"
			});
	});
}

#[test]
fn transfer_from_self() {
	ExtBuilder::default().with_balances(vec![(Alice.into(), 1000)]).build().execute_with(|| {
		assert_ok!(Assets::force_create(
			RuntimeOrigin::root(),
			TEST_ID.into(),
			Alice.into(),
			true,
			1
		));
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(Alice.into()),
			TEST_ID.into(),
			Alice.into(),
			1000
		));

		precompiles()
			.prepare_test(
				Alice, // Alice sending transferFrom herself, no need for allowance.
				Precompile,
				InternalCall::transfer_from {
					from: Address(Alice.into()),
					to: Address(Bob.into()),
					value: 400.into(),
				},
			)
			.expect_log(log3(
				Precompile,
				SELECTOR_LOG_TRANSFER,
				H256::from(Alice),
				H256::from(Bob),
				solidity::encode_event_data(U256::from(400)),
			))
			.execute_returns(true);

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				InternalCall::balance_of { who: Address(Alice.into()) },
			)
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns(U256::from(600));

		precompiles()
			.prepare_test(Alice, Precompile, InternalCall::balance_of { who: Address(Bob.into()) })
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns(U256::from(400));
	});
}

#[test]
fn get_metadata() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000), (Bob.into(), 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				TEST_ID.into(),
				Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::force_set_metadata(
				RuntimeOrigin::root(),
				TEST_ID.into(),
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));

			precompiles()
				.prepare_test(Alice, Precompile, InternalCall::name {})
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(UnboundedBytes::from("TestToken"));

			precompiles()
				.prepare_test(Alice, Precompile, InternalCall::symbol {})
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(UnboundedBytes::from("Test"));

			precompiles()
				.prepare_test(Alice, Precompile, InternalCall::decimals {})
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(12u8);
		});
}

#[test]
fn mint() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000), (Bob.into(), 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				TEST_ID.into(),
				Alice.into(),
				true,
				1
			));

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					InternalCall::mint { to: Address(Bob.into()), value: 400.into() },
				)
				.expect_log(log3(
					Precompile,
					SELECTOR_LOG_TRANSFER,
					H256::default(),
					H256::from(Bob),
					solidity::encode_event_data(U256::from(400)),
				))
				.execute_returns(true);

			precompiles()
				.prepare_test(
					Bob,
					Precompile,
					InternalCall::balance_of { who: Address(Bob.into()) },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(U256::from(400));
		});
}

#[test]
fn burn() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000), (Bob.into(), 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				TEST_ID.into(),
				Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(Alice.into()),
				TEST_ID.into(),
				Alice.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					InternalCall::burn { from: Address(Alice.into()), value: 400.into() },
				)
				.expect_log(log3(
					Precompile,
					SELECTOR_LOG_TRANSFER,
					H256::from(Alice),
					H256::default(),
					solidity::encode_event_data(U256::from(400)),
				))
				.execute_returns(true);

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					InternalCall::balance_of { who: Address(Alice.into()) },
				)
				.expect_no_logs()
				.execute_returns(U256::from(600));
		});
}

#[test]
fn freeze() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000), (Bob.into(), 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				TEST_ID.into(),
				Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(Alice.into()),
				TEST_ID.into(),
				Bob.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					InternalCall::freeze { account: Address(Bob.into()) },
				)
				.expect_no_logs()
				.execute_returns(true);

			precompiles()
				.prepare_test(
					Bob,
					Precompile,
					InternalCall::transfer { to: Address(Alice.into()), value: 400.into() },
				)
				.execute_reverts(|output| {
					from_utf8(output).unwrap().contains("Dispatched call failed with error: ")
						&& from_utf8(output).unwrap().contains("Frozen")
				});
		});
}

#[test]
fn thaw() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000), (Bob.into(), 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				TEST_ID.into(),
				Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(Alice.into()),
				TEST_ID.into(),
				Bob.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					InternalCall::freeze { account: Address(Bob.into()) },
				)
				.expect_no_logs()
				.execute_returns(true);

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					InternalCall::thaw { account: Address(Bob.into()) },
				)
				.expect_no_logs()
				.execute_returns(true);

			precompiles()
				.prepare_test(
					Bob,
					Precompile,
					InternalCall::transfer { to: Address(Alice.into()), value: 400.into() },
				)
				.expect_log(log3(
					Precompile,
					SELECTOR_LOG_TRANSFER,
					H256::from(Bob),
					H256::from(Alice),
					solidity::encode_event_data(U256::from(400)),
				))
				.execute_returns(true);
		});
}

#[test]
fn transfer_ownership() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000), (Bob.into(), 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				TEST_ID.into(),
				Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::force_set_metadata(
				RuntimeOrigin::root(),
				TEST_ID.into(),
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					InternalCall::transfer_ownership { owner: Address(Bob.into()) },
				)
				.expect_no_logs()
				.execute_returns(true);

			// Now Bob should be able to change ownership, and not Alice
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					InternalCall::transfer_ownership { owner: Address(Bob.into()) },
				)
				.execute_reverts(|output| {
					from_utf8(output).unwrap().contains("Dispatched call failed with error: ")
						&& from_utf8(output).unwrap().contains("NoPermission")
				});

			precompiles()
				.prepare_test(
					Bob,
					Precompile,
					InternalCall::transfer_ownership { owner: Address(Alice.into()) },
				)
				.expect_no_logs()
				.execute_returns(true);
		});
}

#[test]
fn transfer_amount_overflow() {
	ExtBuilder::default().with_balances(vec![(Alice.into(), 1000)]).build().execute_with(|| {
		assert_ok!(Assets::force_create(
			RuntimeOrigin::root(),
			TEST_ID.into(),
			Alice.into(),
			true,
			1
		));
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(Alice.into()),
			TEST_ID.into(),
			Alice.into(),
			1000
		));

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				InternalCall::transfer {
					to: Address(Bob.into()),
					value: U256::from(u128::MAX) + 1,
				},
			)
			.expect_no_logs()
			.execute_reverts(|e| e == b"value: Value is too large for balance type");

		precompiles()
			.prepare_test(Bob, Precompile, InternalCall::balance_of { who: Address(Bob.into()) })
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns(U256::from(0));

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				InternalCall::balance_of { who: Address(Alice.into()) },
			)
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns(U256::from(1000));
	});
}

#[test]
fn mint_overflow() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000), (Bob.into(), 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				TEST_ID.into(),
				Alice.into(),
				true,
				1
			));

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					InternalCall::mint {
						to: Address(Bob.into()),
						value: U256::from(u128::MAX) + 1,
					},
				)
				.expect_no_logs()
				.execute_reverts(|e| e == b"value: Value is too large for balance type");
		});
}

#[test]
fn transfer_from_overflow() {
	ExtBuilder::default().with_balances(vec![(Alice.into(), 1000)]).build().execute_with(|| {
		assert_ok!(Assets::force_create(
			RuntimeOrigin::root(),
			TEST_ID.into(),
			Alice.into(),
			true,
			1
		));
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(Alice.into()),
			TEST_ID.into(),
			Alice.into(),
			1000
		));

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				InternalCall::approve { spender: Address(Bob.into()), value: 500.into() },
			)
			.execute_some();

		precompiles()
			.prepare_test(
				Bob, // Bob is the one sending transferFrom!
				Precompile,
				InternalCall::transfer_from {
					from: Address(Alice.into()),
					to: Address(Charlie.into()),
					value: U256::from(u128::MAX) + 1,
				},
			)
			.expect_no_logs()
			.execute_reverts(|e| e == b"value: Value is too large for balance type");
	});
}

#[test]
fn burn_overflow() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000), (Bob.into(), 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				RuntimeOrigin::root(),
				TEST_ID.into(),
				Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(Alice.into()),
				TEST_ID.into(),
				Alice.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					InternalCall::burn {
						from: Address(Alice.into()),
						value: U256::from(u128::MAX) + 1,
					},
				)
				.expect_no_logs()
				.execute_reverts(|e| e == b"value: Value is too large for balance type");
		});
}
