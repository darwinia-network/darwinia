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

// darwinia
use crate::{mock::*, tests::*, RuntimeEthOrigin};
use ethereum::LegacyTransaction;
// frontier
use fp_evm::FeeCalculator;
// substrate
use frame_support::{assert_err, assert_ok, traits::Currency};
use sp_core::U256;
use sp_runtime::{DispatchError, ModuleError};

pub fn legacy_erc20_creation_unsigned_transaction() -> LegacyUnsignedTransaction {
	LegacyUnsignedTransaction {
		nonce: U256::zero(),
		gas_price: U256::from(1),
		gas_limit: U256::from(1_000_000),
		action: ethereum::TransactionAction::Create,
		value: U256::zero(),
		input: array_bytes::hex2bytes_unchecked(ERC20_CONTRACT_BYTECODE),
	}
}

#[test]
fn test_legacy_transaction_works() {
	let alice = address_build(1);
	ExtBuilder::default()
		.with_balances(vec![(alice.address, 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			let t = legacy_erc20_creation_unsigned_transaction().sign(&alice.private_key);

			assert_ok!(RuntimeTransact::runtime_transact(
				RuntimeEthOrigin::RuntimeTransact(alice.address).into(),
				Box::new(t)
			));
			assert!(System::events()
				.iter()
				.any(|record| matches!(record.event, RuntimeEvent::Ethereum(..))));
		});
}

#[test]
fn test_legacy_transaction_with_auto_nonce() {
	let alice = address_build(1);
	ExtBuilder::default()
		.with_balances(vec![(alice.address, 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			let mut unsigned_tx = legacy_erc20_creation_unsigned_transaction();
			unsigned_tx.nonce = U256::MAX;
			let t = unsigned_tx.sign(&alice.private_key);

			assert_ok!(RuntimeTransact::runtime_transact(
				RuntimeEthOrigin::RuntimeTransact(alice.address).into(),
				Box::new(t)
			));
			assert!(System::events()
				.iter()
				.any(|record| matches!(record.event, RuntimeEvent::Ethereum(..))));
		});
}

#[test]
fn test_legacy_transaction_with_auto_gas_price() {
	let alice = address_build(1);
	ExtBuilder::default()
		.with_balances(vec![(alice.address, 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			let mut unsigned_tx = legacy_erc20_creation_unsigned_transaction();
			unsigned_tx.gas_price =
				<Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price().0 - 1;
			let t = unsigned_tx.sign(&alice.private_key);

			assert_ok!(RuntimeTransact::runtime_transact(
				RuntimeEthOrigin::RuntimeTransact(alice.address).into(),
				Box::new(t)
			));
			assert!(System::events()
				.iter()
				.any(|record| matches!(record.event, RuntimeEvent::Ethereum(..))));
		});
}

#[test]
fn test_legacy_transaction_with_sufficient_balance() {
	let alice = address_build(1);
	ExtBuilder::default().build().execute_with(|| {
		let t = legacy_erc20_creation_unsigned_transaction().sign(&alice.private_key);
		assert_err!(
			RuntimeTransact::runtime_transact(
				RuntimeEthOrigin::RuntimeTransact(alice.address).into(),
				Box::new(t.clone())
			),
			DispatchError::Module(ModuleError {
				index: 5,
				error: [0, 4, 0, 0],
				message: Some("MessageTransactError",)
			})
		);

		let fee = RuntimeTransact::total_payment((&t).into());
		let _ = Balances::deposit_creating(&alice.address, fee.as_u64());
		assert_ok!(RuntimeTransact::runtime_transact(
			RuntimeEthOrigin::RuntimeTransact(alice.address).into(),
			Box::new(t)
		));
		assert!(System::events()
			.iter()
			.any(|record| matches!(record.event, RuntimeEvent::Ethereum(..))));
	});
}

#[test]
fn test_legacy_transaction_with_valid_signature() {
	let alice = address_build(1);
	ExtBuilder::default()
		.with_balances(vec![(alice.address, 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			let t = LegacyTransaction {
				nonce: U256::zero(),
				gas_price: U256::from(1),
				gas_limit: U256::from(1_000_000),
				action: ethereum::TransactionAction::Create,
				value: U256::zero(),
				input: array_bytes::hex2bytes_unchecked(ERC20_CONTRACT_BYTECODE),
				signature: TransactionSignature::new(
					38,
					H256([
						190, 103, 224, 160, 125, 182, 125, 168, 212, 70, 247, 106, 221, 89, 14, 84,
						182, 233, 44, 182, 184, 249, 131, 90, 235, 103, 84, 5, 121, 162, 119, 23,
					]),
					H256([
						45, 105, 5, 22, 81, 32, 32, 23, 28, 30, 200, 112, 246, 255, 69, 57, 140,
						200, 96, 146, 80, 50, 107, 232, 153, 21, 251, 83, 142, 123, 215, 24,
					]),
				)
				.unwrap(),
			};
			assert_ok!(RuntimeTransact::runtime_transact(
				RuntimeEthOrigin::RuntimeTransact(alice.address).into(),
				Box::new(Transaction::Legacy(t))
			));

			assert!(System::events()
				.iter()
				.any(|record| matches!(record.event, RuntimeEvent::Ethereum(..))));
		});
}
