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
use crate::{mock::*, tests::*, ForwardEthOrigin};
use ethereum::EIP2930Transaction;
// frontier
use fp_evm::FeeCalculator;
// substrate
use frame_support::{assert_err, assert_ok, traits::Currency};
use sp_core::U256;
use sp_runtime::{DispatchError, ModuleError};

fn eip2930_erc20_creation_unsigned_transaction() -> EIP2930UnsignedTransaction {
	EIP2930UnsignedTransaction {
		nonce: U256::zero(),
		gas_price: U256::from(1),
		gas_limit: U256::from(1_000_000),
		action: ethereum::TransactionAction::Create,
		value: U256::zero(),
		input: array_bytes::hex2bytes_unchecked(ERC20_CONTRACT_BYTECODE),
	}
}

#[test]
fn test_eip2930_transaction_works() {
	let alice = address_build(1);

	ExtBuilder::default()
		.with_balances(vec![(alice.address, 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			let unsigned_tx = eip2930_erc20_creation_unsigned_transaction();
			let t = unsigned_tx.sign(&alice.private_key, None);

			assert_ok!(EthTxForwarder::forward_transact(
				ForwardEthOrigin::ForwardEth(alice.address).into(),
				Box::new(t)
			));
			assert!(System::events()
				.iter()
				.any(|record| matches!(record.event, RuntimeEvent::Ethereum(..))));
		});
}

#[test]
fn test_eip2930_transaction_with_auto_nonce() {
	let alice = address_build(1);

	ExtBuilder::default()
		.with_balances(vec![(alice.address, 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			let mut unsigned_tx = eip2930_erc20_creation_unsigned_transaction();
			unsigned_tx.nonce = U256::MAX;
			let t = unsigned_tx.sign(&alice.private_key, None);

			assert_ok!(EthTxForwarder::forward_transact(
				ForwardEthOrigin::ForwardEth(alice.address).into(),
				Box::new(t)
			));
			assert!(System::events()
				.iter()
				.any(|record| matches!(record.event, RuntimeEvent::Ethereum(..))));
		});
}

#[test]
fn test_eip2930_transaction_with_auto_gas_price() {
	let alice = address_build(1);

	ExtBuilder::default()
		.with_balances(vec![(alice.address, 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			let mut unsigned_tx = eip2930_erc20_creation_unsigned_transaction();
			unsigned_tx.gas_price =
				<Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price().0 - 1;
			let t = unsigned_tx.sign(&alice.private_key, None);
			assert_ok!(EthTxForwarder::forward_transact(
				ForwardEthOrigin::ForwardEth(alice.address).into(),
				Box::new(t)
			));
			assert!(System::events()
				.iter()
				.any(|record| matches!(record.event, RuntimeEvent::Ethereum(..))));
		});
}

#[test]
fn test_eip2930_transaction_with_sufficient_balance() {
	let alice = address_build(1);
	ExtBuilder::default().build().execute_with(|| {
		let unsigned_tx = eip2930_erc20_creation_unsigned_transaction();
		let t = unsigned_tx.sign(&alice.private_key, None);

		assert_err!(
			EthTxForwarder::forward_transact(
				ForwardEthOrigin::ForwardEth(alice.address).into(),
				Box::new(t.clone())
			),
			DispatchError::Module(ModuleError {
				index: 5,
				error: [0, 4, 0, 0],
				message: Some("MessageTransactError",)
			})
		);

		let fee = EthTxForwarder::total_payment((&t).into());
		let _ = Balances::deposit_creating(&alice.address, fee.as_u64());
		assert_ok!(EthTxForwarder::forward_transact(
			ForwardEthOrigin::ForwardEth(alice.address).into(),
			Box::new(t)
		));
		assert!(System::events()
			.iter()
			.any(|record| matches!(record.event, RuntimeEvent::Ethereum(..))));
	});
}

#[test]
fn test_eip2930_transaction_with_valid_signature() {
	let alice = address_build(1);
	ExtBuilder::default()
		.with_balances(vec![(alice.address, 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			let t = EIP2930Transaction {
				chain_id: 0,
				nonce: U256::zero(),
				gas_price: U256::from(1),
				gas_limit: U256::from(1_000_000),
				action: ethereum::TransactionAction::Create,
				value: U256::zero(),
				input: array_bytes::hex2bytes_unchecked(ERC20_CONTRACT_BYTECODE),
				access_list: vec![],
				// copied from:
				// https://github.com/rust-ethereum/ethereum/blob/24739cc8ba6e9d8ee30ada8ec92161e4c48d578e/src/transaction.rs#L873-L875
				odd_y_parity: false,
				// 36b241b061a36a32ab7fe86c7aa9eb592dd59018cd0443adc0903590c16b02b0
				r: H256([
					54, 178, 65, 176, 97, 163, 106, 50, 171, 127, 232, 108, 122, 169, 235, 89, 45,
					213, 144, 24, 205, 4, 67, 173, 192, 144, 53, 144, 193, 107, 2, 176,
				]),
				// 5edcc541b4741c5cc6dd347c5ed9577ef293a62787b4510465fadbfe39ee4094
				s: H256([
					54, 178, 65, 176, 97, 163, 106, 50, 171, 127, 232, 108, 122, 169, 235, 89, 45,
					213, 144, 24, 205, 4, 67, 173, 192, 144, 53, 144, 193, 107, 2, 176,
				]),
			};
			assert_ok!(EthTxForwarder::forward_transact(
				ForwardEthOrigin::ForwardEth(alice.address).into(),
				Box::new(Transaction::EIP2930(t))
			));

			assert!(System::events()
				.iter()
				.any(|record| matches!(record.event, RuntimeEvent::Ethereum(..))));
		});
}
