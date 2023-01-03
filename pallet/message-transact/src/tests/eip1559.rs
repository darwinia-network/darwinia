// This file is part of Darwinia.
//
// Copyright (C) 2018-2023 Darwinia Network
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

// crates
use array_bytes::hex2bytes;
// darwinia
use crate::{mock::*, tests::*};
// frontier
use fp_evm::FeeCalculator;
// substrate
use frame_support::pallet_prelude::Weight;
use sp_core::U256;
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidityError};

pub fn eip1559_erc20_creation_unsigned_transaction() -> EIP1559UnsignedTransaction {
	EIP1559UnsignedTransaction {
		nonce: U256::zero(),
		max_priority_fee_per_gas: U256::from(1),
		max_fee_per_gas: U256::from(1),
		gas_limit: U256::from(1_000_000),
		action: ethereum::TransactionAction::Create,
		value: U256::zero(),
		input: hex2bytes(ERC20_CONTRACT_BYTECODE).unwrap(),
	}
}

#[test]
fn test_dispatch_eip1559_transaction_works() {
	let alice = address_build(1);
	let relayer_account = address_build(2);

	ExtBuilder::default()
		.with_balances(vec![
			(alice.address, 1_000_000_000_000),
			(relayer_account.address, 1_000_000_000_000),
		])
		.build()
		.execute_with(|| {
			let mock_message_id = [0; 4];
			let unsigned_tx = eip1559_erc20_creation_unsigned_transaction();
			let t = unsigned_tx.sign(&alice.private_key, None);
			let call = RuntimeCall::MessageTransact(crate::Call::message_transact {
				transaction: Box::new(t),
			});
			let message = prepare_message(call);

			let result = Dispatch::dispatch(
				SOURCE_CHAIN_ID,
				TARGET_CHAIN_ID,
				&relayer_account.address,
				mock_message_id,
				Ok(message),
				|_, _| Ok(()),
			);

			assert!(result.dispatch_result);
			System::assert_has_event(RuntimeEvent::Dispatch(
				pallet_bridge_dispatch::Event::MessageDispatched(
					SOURCE_CHAIN_ID,
					mock_message_id,
					Ok(()),
				),
			));
		});
}

#[test]
fn test_dispatch_eip1559_transaction_weight_mismatch() {
	let alice = address_build(1);
	let relayer_account = address_build(2);

	ExtBuilder::default()
		.with_balances(vec![
			(alice.address, 1_000_000_000_000),
			(relayer_account.address, 1_000_000_000_000),
		])
		.build()
		.execute_with(|| {
			let mock_message_id = [0; 4];
			let mut unsigned_tx = eip1559_erc20_creation_unsigned_transaction();
			// 62500001 * 16000 > 1_000_000_000_000
			unsigned_tx.gas_limit = U256::from(62500001);
			let t = unsigned_tx.sign(&alice.private_key, None);
			let call = RuntimeCall::MessageTransact(crate::Call::message_transact {
				transaction: Box::new(t),
			});
			let message = prepare_message(call);

			let result = Dispatch::dispatch(
				SOURCE_CHAIN_ID,
				TARGET_CHAIN_ID,
				&relayer_account.address,
				mock_message_id,
				Ok(message),
				|_, _| Ok(()),
			);

			assert!(!result.dispatch_result);
			System::assert_has_event(RuntimeEvent::Dispatch(
				pallet_bridge_dispatch::Event::MessageWeightMismatch(
					SOURCE_CHAIN_ID,
					mock_message_id,
					Weight::from_ref_time(1249913722000),
					Weight::from_ref_time(1000000000000),
				),
			));
		});
}

#[test]
fn test_dispatch_eip1559_transaction_with_autoset_nonce() {
	let alice = address_build(1);
	let relayer = address_build(2);

	ExtBuilder::default()
		.with_balances(vec![
			(alice.address, 1_000_000_000_000),
			(relayer.address, 1_000_000_000_000),
		])
		.build()
		.execute_with(|| {
			let mock_message_id = [0; 4];
			let mut unsigned_tx = eip1559_erc20_creation_unsigned_transaction();
			unsigned_tx.nonce = U256::MAX;
			let t = unsigned_tx.sign(&alice.private_key, None);
			let call = RuntimeCall::MessageTransact(crate::Call::message_transact {
				transaction: Box::new(t),
			});
			let message = prepare_message(call);

			let result = Dispatch::dispatch(
				SOURCE_CHAIN_ID,
				TARGET_CHAIN_ID,
				&relayer.address,
				mock_message_id,
				Ok(message),
				|_, _| Ok(()),
			);

			assert!(result.dispatch_result);
		});
}

#[test]
fn test_dispatch_eip1559_transaction_with_autoset_gas_price() {
	let alice = address_build(1);
	let relayer = address_build(2);

	ExtBuilder::default()
		.with_balances(vec![
			(alice.address, 1_000_000_000_000),
			(relayer.address, 1_000_000_000_000),
		])
		.build()
		.execute_with(|| {
			let mock_message_id = [0; 4];
			let mut unsigned_tx = eip1559_erc20_creation_unsigned_transaction();
			unsigned_tx.max_fee_per_gas =
				<TestRuntime as pallet_evm::Config>::FeeCalculator::min_gas_price().0 - 1;
			let t = unsigned_tx.sign(&alice.private_key, None);
			let call = RuntimeCall::MessageTransact(crate::Call::message_transact {
				transaction: Box::new(t),
			});
			let message = prepare_message(call);

			let result = Dispatch::dispatch(
				SOURCE_CHAIN_ID,
				TARGET_CHAIN_ID,
				&relayer.address,
				mock_message_id,
				Ok(message),
				|_, _| Ok(()),
			);

			assert!(result.dispatch_result);
		});
}

#[test]
fn test_dispatch_eip1559_transaction_with_insufficient_relayer_balance() {
	let alice = address_build(1);
	let relayer1 = address_build(2);
	let relayer2 = address_build(3);

	ExtBuilder::default()
		.with_balances(vec![
			(alice.address, 1_000_000_000_000),
			(relayer1.address, 1_000),
			(relayer2.address, 1_000_000_000_000),
		])
		.build()
		.execute_with(|| {
			let mock_message_id = [0; 4];
			let unsigned_tx = eip1559_erc20_creation_unsigned_transaction();
			let t = unsigned_tx.sign(&alice.private_key, None);
			let call = RuntimeCall::MessageTransact(crate::Call::message_transact {
				transaction: Box::new(t),
			});
			let message = prepare_message(call);

			// Failed in pre-dispatch balance check
			let before_dispatch =
				pallet_evm::Pallet::<TestRuntime>::account_basic(&relayer1.address).0.balance;
			let result = Dispatch::dispatch(
				SOURCE_CHAIN_ID,
				TARGET_CHAIN_ID,
				&relayer1.address,
				mock_message_id,
				Ok(message.clone()),
				|_, _| Ok(()),
			);
			assert!(!result.dispatch_result);
			System::assert_has_event(RuntimeEvent::Dispatch(
				pallet_bridge_dispatch::Event::MessageCallValidateFailed(
					SOURCE_CHAIN_ID,
					mock_message_id,
					TransactionValidityError::Invalid(InvalidTransaction::Payment),
				),
			));
			let after_dispatch =
				pallet_evm::Pallet::<TestRuntime>::account_basic(&relayer1.address).0.balance;
			assert_eq!(before_dispatch, after_dispatch);

			let before_dispatch =
				pallet_evm::Pallet::<TestRuntime>::account_basic(&relayer2.address).0.balance;
			let result = Dispatch::dispatch(
				SOURCE_CHAIN_ID,
				TARGET_CHAIN_ID,
				&relayer2.address,
				mock_message_id,
				Ok(message),
				|_, _| Ok(()),
			);
			assert!(result.dispatch_result);
			let after_dispatch =
				pallet_evm::Pallet::<TestRuntime>::account_basic(&relayer2.address).0.balance;
			assert!(before_dispatch > after_dispatch);
		});
}
