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
use crate::{mock::*, ForwardEthOrigin, ForwardRequest, TxType};
// crates.io
use ethereum::TransactionSignature;
use fp_evm::ExecutionInfoV2;
// polkadot-sdk
use frame_support::{assert_err, assert_ok, traits::Currency};
use sp_core::{H256, U256};
use sp_runtime::{DispatchError, ModuleError};

// This ERC-20 contract mints the maximum amount of tokens to the contract creator.
// pragma solidity ^0.5.0;
// import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v2.5.1/contracts/token/ERC20/ERC20.sol";
// contract MyToken is ERC20 {
//	 constructor() public { _mint(msg.sender, 1000000); }
// }
const ERC20_CONTRACT_BYTECODE: &str = "608060405234801561001057600080fd5b5061002433620f424061002960201b60201c565b610274565b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614156100cc576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040180806020018281038252601f8152602001807f45524332303a206d696e7420746f20746865207a65726f20616464726573730081525060200191505060405180910390fd5b6100e5816002546101ec60201b610c7c1790919060201c565b600281905550610140816000808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020546101ec60201b610c7c1790919060201c565b6000808473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055508173ffffffffffffffffffffffffffffffffffffffff16600073ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef836040518082815260200191505060405180910390a35050565b60008082840190508381101561026a576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040180806020018281038252601b8152602001807f536166654d6174683a206164646974696f6e206f766572666c6f77000000000081525060200191505060405180910390fd5b8091505092915050565b610e3a806102836000396000f3fe608060405234801561001057600080fd5b50600436106100885760003560e01c806370a082311161005b57806370a08231146101fd578063a457c2d714610255578063a9059cbb146102bb578063dd62ed3e1461032157610088565b8063095ea7b31461008d57806318160ddd146100f357806323b872dd146101115780633950935114610197575b600080fd5b6100d9600480360360408110156100a357600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190505050610399565b604051808215151515815260200191505060405180910390f35b6100fb6103b7565b6040518082815260200191505060405180910390f35b61017d6004803603606081101561012757600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291905050506103c1565b604051808215151515815260200191505060405180910390f35b6101e3600480360360408110156101ad57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff1690602001909291908035906020019092919050505061049a565b604051808215151515815260200191505060405180910390f35b61023f6004803603602081101561021357600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919050505061054d565b6040518082815260200191505060405180910390f35b6102a16004803603604081101561026b57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190505050610595565b604051808215151515815260200191505060405180910390f35b610307600480360360408110156102d157600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190505050610662565b604051808215151515815260200191505060405180910390f35b6103836004803603604081101561033757600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190505050610680565b6040518082815260200191505060405180910390f35b60006103ad6103a6610707565b848461070f565b6001905092915050565b6000600254905090565b60006103ce848484610906565b61048f846103da610707565b61048a85604051806060016040528060288152602001610d7060289139600160008b73ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000206000610440610707565b73ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054610bbc9092919063ffffffff16565b61070f565b600190509392505050565b60006105436104a7610707565b8461053e85600160006104b8610707565b73ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008973ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054610c7c90919063ffffffff16565b61070f565b6001905092915050565b60008060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020549050919050565b60006106586105a2610707565b8461065385604051806060016040528060258152602001610de160259139600160006105cc610707565b73ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008a73ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054610bbc9092919063ffffffff16565b61070f565b6001905092915050565b600061067661066f610707565b8484610906565b6001905092915050565b6000600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054905092915050565b600033905090565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff161415610795576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401808060200182810382526024815260200180610dbd6024913960400191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff16141561081b576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401808060200182810382526022815260200180610d286022913960400191505060405180910390fd5b80600160008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055508173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925836040518082815260200191505060405180910390a3505050565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff16141561098c576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401808060200182810382526025815260200180610d986025913960400191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff161415610a12576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401808060200182810382526023815260200180610d056023913960400191505060405180910390fd5b610a7d81604051806060016040528060268152602001610d4a602691396000808773ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054610bbc9092919063ffffffff16565b6000808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002081905550610b10816000808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054610c7c90919063ffffffff16565b6000808473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055508173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef836040518082815260200191505060405180910390a3505050565b6000838311158290610c69576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825283818151815260200191508051906020019080838360005b83811015610c2e578082015181840152602081019050610c13565b50505050905090810190601f168015610c5b5780820380516001836020036101000a031916815260200191505b509250505060405180910390fd5b5060008385039050809150509392505050565b600080828401905083811015610cfa576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040180806020018281038252601b8152602001807f536166654d6174683a206164646974696f6e206f766572666c6f77000000000081525060200191505060405180910390fd5b809150509291505056fe45524332303a207472616e7366657220746f20746865207a65726f206164647265737345524332303a20617070726f766520746f20746865207a65726f206164647265737345524332303a207472616e7366657220616d6f756e7420657863656564732062616c616e636545524332303a207472616e7366657220616d6f756e74206578636565647320616c6c6f77616e636545524332303a207472616e736665722066726f6d20746865207a65726f206164647265737345524332303a20617070726f76652066726f6d20746865207a65726f206164647265737345524332303a2064656372656173656420616c6c6f77616e63652062656c6f77207a65726fa265627a7a723158204b72fba02adebd751a8e173005e094f50807c69fde771436ef2cb081650def3c64736f6c63430005110032";

fn mocked_request() -> ForwardRequest {
	ForwardRequest {
		tx_type: TxType::default(),
		gas_limit: U256::from(1_000_000),
		action: ethereum::TransactionAction::Create,
		value: U256::zero(),
		input: array_bytes::hex2bytes_unchecked(ERC20_CONTRACT_BYTECODE),
	}
}

#[test]
fn forward_request_works() {
	let alice = address_build(1);
	ExtBuilder::default()
		.with_balances(vec![(alice.address, 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			macro_rules! test_tx_types {
				($tx_type:expr) => {
					let mut request = mocked_request();
					request.tx_type = $tx_type;

					assert_ok!(EthTxForwarder::forward_transact(
						ForwardEthOrigin::ForwardEth(alice.address).into(),
						request,
					));
					assert!(System::events()
						.iter()
						.any(|record| matches!(record.event, RuntimeEvent::Ethereum(..))));
				};
			}

			test_tx_types!(TxType::LegacyTransaction);
			test_tx_types!(TxType::EIP1559Transaction);
			test_tx_types!(TxType::EIP2930Transaction);
		});
}

#[test]
fn forward_request_sufficient_balance() {
	let alice = address_build(1);
	ExtBuilder::default().build().execute_with(|| {
		macro_rules! test_tx_types {
			($tx_type:expr) => {
				let mut request = mocked_request();
				request.tx_type = $tx_type;

				assert_err!(
					EthTxForwarder::forward_transact(
						ForwardEthOrigin::ForwardEth(alice.address).into(),
						request.clone()
					),
					DispatchError::Module(ModuleError {
						index: 5,
						error: [0, 4, 0, 0],
						message: Some("ValidationError",)
					})
				);

				let fee = EthTxForwarder::total_payment(&request);
				let _ = Balances::deposit_creating(&alice.address, fee.as_u64());
				assert_ok!(EthTxForwarder::forward_transact(
					ForwardEthOrigin::ForwardEth(alice.address).into(),
					request
				));
				assert!(System::events()
					.iter()
					.any(|record| matches!(record.event, RuntimeEvent::Ethereum(..))));
			};
		}
		test_tx_types!(TxType::LegacyTransaction);
		test_tx_types!(TxType::EIP1559Transaction);
		test_tx_types!(TxType::EIP2930Transaction);
	});
}

#[test]
fn foraward_call_works() {
	let alice = address_build(1);
	ExtBuilder::default()
		.with_balances(vec![(alice.address, 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			let request = mocked_request();
			assert_ok!(EthTxForwarder::forward_transact(
				ForwardEthOrigin::ForwardEth(alice.address).into(),
				request,
			));
			let pallet_ethereum::Event::Executed { to, .. } =
				System::read_events_for_pallet().into_iter().nth(0).expect("events expected");

			use ethabi::{Function, Param, ParamType, Token};
			#[allow(deprecated)]
			let function = Function {
				name: "balanceOf".to_owned(),
				inputs: vec![Param {
					name: "account".to_owned(),
					kind: ParamType::Address,
					internal_type: None,
				}],
				outputs: vec![Param {
					name: "balance".to_owned(),
					kind: ParamType::Uint(256),
					internal_type: None,
				}],
				constant: None,
				state_mutability: ethabi::StateMutability::View,
			};
			let input = function.encode_input(&[Token::Address(alice.address)]).unwrap();
			let result = EthTxForwarder::forward_call(
				alice.address,
				to,
				input,
				Default::default(),
				U256::from(10_000_000u64),
			)
			.ok()
			.expect("call should succeed");
			let ExecutionInfoV2 { value, .. } = result;
			assert_eq!(U256::from_big_endian(&value), U256::from(1000000));
		});
}

#[test]
fn mock_signature_valid() {
	assert!(
		// copied from:
		// https://github.com/rust-ethereum/ethereum/blob/24739cc8ba6e9d8ee30ada8ec92161e4c48d578e/src/transaction.rs#L798
		TransactionSignature::new(
			38,
			// be67e0a07db67da8d446f76add590e54b6e92cb6b8f9835aeb67540579a27717
			H256([
				190, 103, 224, 160, 125, 182, 125, 168, 212, 70, 247, 106, 221, 89, 14, 84, 182,
				233, 44, 182, 184, 249, 131, 90, 235, 103, 84, 5, 121, 162, 119, 23,
			]),
			// 2d690516512020171c1ec870f6ff45398cc8609250326be89915fb538e7bd718
			H256([
				45, 105, 5, 22, 81, 32, 32, 23, 28, 30, 200, 112, 246, 255, 69, 57, 140, 200, 96,
				146, 80, 50, 107, 232, 153, 21, 251, 83, 142, 123, 215, 24,
			]),
		)
		.is_some(),
	)
}
