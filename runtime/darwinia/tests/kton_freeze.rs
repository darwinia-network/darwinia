// This file is part of Darwinia.
//
// Copyright (C) Darwinia Network
// SPDX-License-Identifier: GPL-3.0

mod mock;

use frame_support::{
	assert_ok,
	traits::{fungibles::Inspect, Contains},
};
use mock::*;
use pallet_assets::Call as AssetCall;
use pallet_evm::Runner;
use sp_core::{H160, U256};
use sp_runtime::traits::Dispatchable;

fn asset_calls(asset: AssetId) -> Vec<RuntimeCall> {
	let id = asset.into();
	let who = ROOT;
	vec![
		AssetCall::create { id, admin: who, min_balance: 1 },
		AssetCall::force_create { id, owner: who, is_sufficient: true, min_balance: 1 },
		AssetCall::start_destroy { id },
		AssetCall::destroy_accounts { id },
		AssetCall::destroy_approvals { id },
		AssetCall::finish_destroy { id },
		AssetCall::mint { id, beneficiary: who, amount: 1 },
		AssetCall::burn { id, who, amount: 1 },
		AssetCall::transfer { id, target: who, amount: 1 },
		AssetCall::transfer_keep_alive { id, target: who, amount: 1 },
		AssetCall::force_transfer { id, source: who, dest: who, amount: 1 },
		AssetCall::freeze { id, who },
		AssetCall::thaw { id, who },
		AssetCall::freeze_asset { id },
		AssetCall::thaw_asset { id },
		AssetCall::transfer_ownership { id, owner: who },
		AssetCall::set_team { id, issuer: who, admin: who, freezer: who },
		AssetCall::set_metadata { id, name: vec![], symbol: vec![], decimals: 18 },
		AssetCall::clear_metadata { id },
		AssetCall::force_set_metadata {
			id,
			name: vec![],
			symbol: vec![],
			decimals: 18,
			is_frozen: false,
		},
		AssetCall::force_clear_metadata { id },
		AssetCall::force_asset_status {
			id,
			owner: who,
			issuer: who,
			admin: who,
			freezer: who,
			min_balance: 1,
			is_sufficient: true,
			is_frozen: false,
		},
		AssetCall::approve_transfer { id, delegate: who, amount: 1 },
		AssetCall::cancel_approval { id, delegate: who },
		AssetCall::force_cancel_approval { id, owner: who, delegate: who },
		AssetCall::transfer_approved { id, owner: who, destination: who, amount: 1 },
		AssetCall::touch { id },
		AssetCall::refund { id, allow_burn: true },
		AssetCall::set_min_balance { id, min_balance: 1 },
		AssetCall::touch_other { id, who },
		AssetCall::refund_other { id, who },
		AssetCall::block { id, who },
		AssetCall::transfer_all { id, dest: who, keep_alive: false },
	]
	.into_iter()
	.map(RuntimeCall::Assets)
	.collect()
}

fn ext() -> sp_io::TestExternalities {
	ExtBuilder::default()
		.with_balances(vec![(ROOT, 1_000 * UNIT)])
		.with_assets_accounts(vec![(KTON_ID, ROOT, 100)])
		.build()
}

fn transfer() -> RuntimeCall {
	RuntimeCall::Assets(AssetCall::transfer {
		id: KTON_ID.into(),
		target: H160::repeat_byte(0xBB).into(),
		amount: 10,
	})
}

#[test]
fn every_kton_asset_call_is_filtered_but_other_assets_are_allowed() {
	ext().execute_with(|| {
		for call in asset_calls(KTON_ID) {
			assert!(!KtonCallFilter::contains(&call));
			let result = call.dispatch(RuntimeOrigin::signed(ROOT));
			assert_eq!(
				result.unwrap_err().error,
				frame_system::Error::<Runtime>::CallFiltered.into()
			);
		}
		for call in asset_calls(KTON_ID + 1) {
			assert!(KtonCallFilter::contains(&call));
		}
		assert_eq!(Assets::balance(KTON_ID, ROOT), 100);
		assert_eq!(Assets::total_issuance(KTON_ID), 100);
	});
}

#[test]
fn non_kton_transfers_continue_to_work() {
	ext().execute_with(|| {
		let id = KTON_ID + 1;
		let recipient = H160::repeat_byte(0xBB).into();
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), id.into(), ROOT, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(ROOT), id.into(), ROOT, 100));
		let call = RuntimeCall::Assets(AssetCall::transfer {
			id: id.into(),
			target: recipient,
			amount: 10,
		});
		assert_ok!(call.dispatch(RuntimeOrigin::signed(ROOT)));
		assert_eq!(Assets::balance(id, recipient), 10);
		let ring = RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death {
			dest: recipient,
			value: UNIT,
		});
		assert_ok!(ring.dispatch(RuntimeOrigin::signed(ROOT)));
		assert_eq!(Balances::free_balance(recipient), UNIT);
	});
}

#[test]
fn batch_proxy_and_derivative_calls_cannot_move_kton() {
	ext().execute_with(|| {
		let batch = RuntimeCall::Utility(pallet_utility::Call::batch { calls: vec![transfer()] });
		assert_ok!(batch.dispatch(RuntimeOrigin::signed(ROOT)));
		assert!(System::events().iter().any(|record| matches!(
			record.event,
			RuntimeEvent::Utility(pallet_utility::Event::BatchInterrupted { index: 0, .. })
		)));
		let batch_all =
			RuntimeCall::Utility(pallet_utility::Call::batch_all { calls: vec![transfer()] });
		assert!(batch_all.dispatch(RuntimeOrigin::signed(ROOT)).is_err());
		let derivative = RuntimeCall::Utility(pallet_utility::Call::as_derivative {
			index: 0,
			call: Box::new(transfer()),
		});
		assert_eq!(
			derivative.dispatch(RuntimeOrigin::signed(ROOT)).unwrap_err().error,
			frame_system::Error::<Runtime>::CallFiltered.into()
		);

		let delegate = H160::repeat_byte(0xCC).into();
		assert_ok!(Proxy::add_proxy(RuntimeOrigin::signed(ROOT), delegate, ProxyType::Any, 0));
		let proxy = RuntimeCall::Proxy(pallet_proxy::Call::proxy {
			real: ROOT,
			force_proxy_type: None,
			call: Box::new(transfer()),
		});
		assert_ok!(proxy.dispatch(RuntimeOrigin::signed(delegate)));
		assert!(System::events().iter().any(|record| matches!(
			record.event,
			RuntimeEvent::Proxy(pallet_proxy::Event::ProxyExecuted { result: Err(_) })
		)));
		assert_eq!(Assets::balance(KTON_ID, ROOT), 100);
		assert_eq!(Assets::total_issuance(KTON_ID), 100);
	});
}

fn abi(selector: [u8; 4], args: &[U256]) -> Vec<u8> {
	let mut data = selector.to_vec();
	for arg in args {
		let mut word = [0; 32];
		arg.to_big_endian(&mut word);
		data.extend_from_slice(&word);
	}
	data
}

fn address_arg(address: H160) -> U256 {
	U256::from_big_endian(address.as_bytes())
}

fn evm_call(caller: H160, target: H160, data: Vec<u8>) -> fp_evm::CallInfo {
	<Runtime as pallet_evm::Config>::Runner::call(
		caller,
		target,
		data,
		U256::zero(),
		1_000_000,
		None,
		None,
		None,
		Vec::new(),
		false,
		false,
		None,
		None,
		<Runtime as pallet_evm::Config>::config(),
	)
	.expect("EVM execution must reach the target")
}

#[test]
fn existing_erc20_approvals_do_not_bypass_the_runtime_filter() {
	ext().execute_with(|| {
		let owner: H160 = ROOT.into();
		let spender = H160::repeat_byte(0xCC);
		let recipient = H160::repeat_byte(0xBB);
		let token = H160::from_low_u64_be(KTON_ID);
		// Set up an allowance that existed before the migration.
		assert_ok!(Assets::approve_transfer(
			RuntimeOrigin::signed(ROOT),
			KTON_ID.into(),
			spender.into(),
			50
		));
		let attempts = [
			(owner, abi([0xa9, 0x05, 0x9c, 0xbb], &[address_arg(recipient), 10.into()])),
			(
				spender,
				abi(
					[0x23, 0xb8, 0x72, 0xdd],
					&[address_arg(owner), address_arg(recipient), 10.into()],
				),
			),
			(
				owner,
				abi(
					[0x23, 0xb8, 0x72, 0xdd],
					&[address_arg(owner), address_arg(recipient), 10.into()],
				),
			),
			(owner, abi([0x40, 0xc1, 0x0f, 0x19], &[address_arg(recipient), 10.into()])),
			(owner, abi([0x9d, 0xc2, 0x9f, 0xac], &[address_arg(owner), 10.into()])),
			(owner, abi([0x09, 0x5e, 0xa7, 0xb3], &[address_arg(spender), 10.into()])),
		];
		for (caller, data) in attempts {
			let result = evm_call(caller, token, data);
			assert!(matches!(result.exit_reason, fp_evm::ExitReason::Revert(_)));
			assert!(result
				.value
				.windows(b"CallFiltered".len())
				.any(|word| word == b"CallFiltered"));
		}
		assert_eq!(Assets::balance(KTON_ID, ROOT), 100);
		assert_eq!(Assets::balance(KTON_ID, AccountId::from(recipient)), 0);
		assert_eq!(Assets::total_issuance(KTON_ID), 100);
		assert_eq!(
			<Assets as frame_support::traits::fungibles::approvals::Inspect<AccountId>>::allowance(
				KTON_ID,
				&ROOT,
				&spender.into()
			),
			50
		);
	});
}

#[test]
fn erc20_balance_supply_and_metadata_reads_remain_available() {
	ext().execute_with(|| {
		let owner: H160 = ROOT.into();
		let token = H160::from_low_u64_be(KTON_ID);
		// Exercise the actual Executive hook registration, not just the pallet API.
		darwinia_staking::CollatorCount::<Runtime>::put(17);
		Executive::execute_on_runtime_upgrade();
		assert_eq!(darwinia_staking::CollatorCount::<Runtime>::get(), 17);
		assert!(Assets::transfer(
			RuntimeOrigin::signed(ROOT),
			KTON_ID.into(),
			H160::repeat_byte(0xBB).into(),
			1
		)
		.is_err());
		for data in [
			abi([0x70, 0xa0, 0x82, 0x31], &[address_arg(owner)]),
			abi([0x18, 0x16, 0x0d, 0xdd], &[]),
		] {
			let result = evm_call(owner, token, data);
			assert!(result.exit_reason.is_succeed());
			assert_eq!(U256::from_big_endian(&result.value), U256::from(100));
		}
		for selector in
			[[0x06, 0xfd, 0xde, 0x03], [0x95, 0xd8, 0x9b, 0x41], [0x31, 0x3c, 0xe5, 0x67]]
		{
			assert!(evm_call(owner, token, abi(selector, &[])).exit_reason.is_succeed());
		}
	});
}

#[test]
fn other_asset_precompiles_still_allow_transfers_after_the_upgrade() {
	ext().execute_with(|| {
		let id = KTON_ID + 1;
		let recipient = H160::repeat_byte(0xBB);
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), id.into(), ROOT, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(ROOT), id.into(), ROOT, 100));
		Executive::execute_on_runtime_upgrade();
		let result = evm_call(
			ROOT.into(),
			H160::from_low_u64_be(id),
			abi([0xa9, 0x05, 0x9c, 0xbb], &[address_arg(recipient), 10.into()]),
		);
		assert!(result.exit_reason.is_succeed());
		assert_eq!(Assets::balance(id, AccountId::from(recipient)), 10);
		assert_eq!(Assets::balance(id, ROOT), 90);
		assert_eq!(Assets::total_issuance(id), 100);
	});
}

#[test]
fn ordinary_thawing_is_blocked_but_governance_bypass_remains_explicit() {
	use frame_support::traits::UnfilteredDispatchable;

	ext().execute_with(|| {
		Executive::execute_on_runtime_upgrade();
		let thaw = RuntimeCall::Assets(AssetCall::thaw_asset { id: KTON_ID.into() });
		assert_eq!(
			thaw.clone().dispatch(RuntimeOrigin::signed(ROOT)).unwrap_err().error,
			frame_system::Error::<Runtime>::CallFiltered.into()
		);
		// Match FRAME's existing trusted-governance boundary; this filter is not a
		// promise that Root or explicitly unfiltered dispatch can never thaw KTON.
		assert_ok!(thaw.dispatch_bypass_filter(RuntimeOrigin::signed(ROOT)));
		assert_ok!(Assets::transfer(
			RuntimeOrigin::signed(ROOT),
			KTON_ID.into(),
			H160::repeat_byte(0xBB).into(),
			1
		));
		// A normal user call is still filtered even if governance changes the status.
		assert!(transfer().dispatch(RuntimeOrigin::signed(ROOT)).is_err());
	});
}

#[test]
fn contract_call_failure_rolls_back_its_own_bookkeeping() {
	ext().execute_with(|| {
		let caller: H160 = ROOT.into();
		let contract = H160::repeat_byte(0xDD);
		let recipient = H160::repeat_byte(0xBB);
		// A minimal staking-like contract: change a storage slot, call KTON transfer,
		// and bubble up a failed call. Its bookkeeping must revert with the transfer.
		let mut code = vec![0x60, 0x01, 0x60, 0x00, 0x55]; // SSTORE(0, 1)
		code.extend_from_slice(&[0x36, 0x60, 0x00, 0x60, 0x00, 0x37]); // copy calldata
		code.extend_from_slice(&[0x60, 0x00, 0x60, 0x00, 0x36, 0x60, 0x00, 0x60, 0x00, 0x73]);
		code.extend_from_slice(H160::from_low_u64_be(KTON_ID).as_bytes());
		code.extend_from_slice(&[0x5a, 0xf1, 0x15]); // CALL; ISZERO
		code.extend_from_slice(&[0x60, 0x00, 0x57, 0x00]);
		let destination = code.len() as u8;
		let destination_index = code.len() - 3;
		code[destination_index] = destination;
		code.extend_from_slice(&[0x5b, 0x60, 0x00, 0x60, 0x00, 0xfd]);
		pallet_evm::AccountCodes::<Runtime>::insert(contract, code);
		// Existing principal held by the contract before the upgrade.
		assert_ok!(Assets::mint(RuntimeOrigin::signed(ROOT), KTON_ID.into(), contract.into(), 25));
		let result = evm_call(
			caller,
			contract,
			abi([0xa9, 0x05, 0x9c, 0xbb], &[address_arg(recipient), 10.into()]),
		);
		assert!(matches!(result.exit_reason, fp_evm::ExitReason::Revert(_)));
		assert_eq!(
			pallet_evm::AccountStorages::<Runtime>::get(contract, sp_core::H256::zero()),
			sp_core::H256::zero()
		);
		assert_eq!(Assets::balance(KTON_ID, AccountId::from(contract)), 25);
		assert_eq!(Assets::balance(KTON_ID, AccountId::from(recipient)), 0);
		assert_eq!(Assets::total_issuance(KTON_ID), 125);
	});
}
