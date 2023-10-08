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

pub mod mock;

darwinia_common_runtime::impl_weight_tests! {}
darwinia_common_runtime::impl_fee_tests! {}
// darwinia_common_runtime::impl_evm_tests! {}
darwinia_common_runtime::impl_account_migration_tests! {}
darwinia_common_runtime::impl_messages_bridge_tests! {}

#[test]
fn precompile_address() {
	use crate::mock::*;

	assert_eq!(
		PangolinPrecompiles::<Runtime>::used_addresses()
			.iter()
			.map(|a| a.to_low_u64_be())
			.collect::<Vec<u64>>(),
		vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 1024, 1025, 1026, 1027, 1536, 1537, 2048]
	);
}

mod message_transact {
	// darwinia
	use super::mock::*;
	use frame_support::traits::EnsureOrigin;
	use sp_core::H160;
	use static_assertions::assert_type_eq_all;

	#[test]
	fn ensure_origin_correctly() {
		assert_type_eq_all!(
			<<Runtime as darwinia_message_transact::Config>::LcmpEthOrigin as EnsureOrigin<
				RuntimeOrigin,
			>>::Success,
			H160
		);
	}
}

mod balances {
	// darwinia
	use super::mock::*;
	use frame_support::traits::Get;

	#[test]
	fn ensure_constants_correctly() {
		assert_eq!(<Runtime as pallet_balances::Config>::ExistentialDeposit::get(), 0);
		assert_eq!(<<Runtime as pallet_balances::Config>::MaxLocks as Get<u32>>::get(), 50);
		assert_eq!(<<Runtime as pallet_balances::Config>::MaxReserves as Get<u32>>::get(), 50);
	}
}

mod evm {
	// darwinia
	use super::mock::*;
	// frontier
	use pallet_ethereum::PostLogContent;
	use pallet_evm_precompile_dispatch::DispatchValidateT;
	// substrate
	use frame_support::{assert_err, traits::Get};
	use sp_core::{H160, U256};
	use sp_runtime::{DispatchError, ModuleError};

	#[test]
	fn configured_base_extrinsic_weight_is_evm_compatible() {
		let min_ethereum_transaction_weight = WeightPerGas::get() * 21_000;
		let base_extrinsic = <Runtime as frame_system::Config>::BlockWeights::get()
			.get(frame_support::dispatch::DispatchClass::Normal)
			.base_extrinsic;

		assert!(base_extrinsic.ref_time() <= min_ethereum_transaction_weight.ref_time());
	}

	fn ethereum_constants_are_correctly() {
		assert_eq!(<<Runtime as pallet_ethereum::Config>::ExtraDataLength as Get<u32>>::get(), 64);
		assert_eq!(
			<Runtime as pallet_ethereum::Config>::PostLogContent::get() as u8,
			PostLogContent::BlockAndTxnHashes as u8
		);
	}

	#[test]
	fn evm_constants_are_correctly() {
		assert_eq!(BlockGasLimit::get(), U256::from(20_000_000));
		assert_eq!(WeightPerGas::get().ref_time(), 18750);
		assert_eq!(GasLimitPovSizeRatio::get(), 6);
	}

	#[test]
	fn pallet_evm_calls_only_callable_by_root() {
		ExtBuilder::default().build().execute_with(|| {
			// https://github.com/darwinia-network/darwinia/blob/5923b2e0526b67fe05cee6e4e592ceca80e8f2ff/runtime/darwinia/src/pallets/evm.rs#L136
			assert_err!(
				EVM::call(
					RuntimeOrigin::signed(H160::default().into()),
					H160::default(),
					H160::default(),
					vec![],
					U256::default(),
					1000000,
					U256::from(1_000_000),
					None,
					None,
					vec![],
				),
				DispatchError::BadOrigin
			);

			if let Err(dispatch_info_with_err) = EVM::call(
				RuntimeOrigin::root(),
				H160::default(),
				H160::default(),
				vec![],
				U256::default(),
				1000000,
				U256::from(1_000_000),
				None,
				None,
				vec![],
			) {
				assert_eq!(
					dispatch_info_with_err.error,
					DispatchError::Module(ModuleError {
						index: 37,
						error: [4, 0, 0, 0],
						message: Some("GasPriceTooLow")
					})
				);
			}
		});
	}

	#[test]
	fn dispatch_validator_filter_runtime_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert!(DarwiniaDispatchValidator::validate_before_dispatch(
				&H160::default().into(),
				&RuntimeCall::System(frame_system::Call::remark { remark: vec![] })
			)
			.is_none());

			assert!(DarwiniaDispatchValidator::validate_before_dispatch(
				&H160::default().into(),
				// forbidden call
				&RuntimeCall::EVM(pallet_evm::Call::call {
					source: H160::default(),
					target: H160::default(),
					input: vec![],
					value: U256::default(),
					gas_limit: 1000000,
					max_fee_per_gas: U256::from(1_000_000),
					max_priority_fee_per_gas: None,
					nonce: None,
					access_list: vec![],
				})
			)
			.is_some());
		});
	}

	#[test]
	fn dispatch_validator_filter_dispatch_class() {
		ExtBuilder::default().build().execute_with(|| {
			// Default class
			assert!(DarwiniaDispatchValidator::validate_before_dispatch(
				&H160::default().into(),
				&RuntimeCall::System(frame_system::Call::remark { remark: vec![] })
			)
			.is_none());

			// Operational class
			assert!(DarwiniaDispatchValidator::validate_before_dispatch(
				&H160::default().into(),
				&RuntimeCall::System(frame_system::Call::set_heap_pages { pages: 20 })
			)
			.is_none());

			// Mandatory class
			assert!(DarwiniaDispatchValidator::validate_before_dispatch(
				&H160::default().into(),
				&RuntimeCall::Timestamp(pallet_timestamp::Call::set { now: 100 })
			)
			.is_some());
		});
	}
}
