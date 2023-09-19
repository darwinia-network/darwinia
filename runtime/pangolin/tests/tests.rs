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

// darwinia_common_runtime::impl_weight_tests! {}
// darwinia_common_runtime::impl_fee_tests! {}
// darwinia_common_runtime::impl_evm_tests! {}
// darwinia_common_runtime::impl_account_migration_tests! {}
// darwinia_common_runtime::impl_messages_bridge_tests! {}

mod evm {
	// darwinia
	use super::mock::*;
	// frontier
	use pallet_evm_precompile_dispatch::DispatchValidateT;
	// substrate
	use frame_support::{assert_err, pallet_prelude::DispatchClass};
	use pallet_evm::GasWeightMapping;
	use sp_core::{H160, U256};
	use sp_runtime::{DispatchError, ModuleError};

	#[test]
	fn evm_constants_are_correctly() {
		let w1 = <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(12_000_000, true);
		let max1 = <Runtime as frame_system::Config>::BlockWeights::get()
			.get(DispatchClass::Normal)
			.max_extrinsic
			.unwrap();
		println!("1200 => w: {:?}", w1);
		println!("1200 => max: {:?}", max1);

		let w2 = <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(14_000_000, true);
		let max2 = <Runtime as frame_system::Config>::BlockWeights::get()
			.get(DispatchClass::Normal)
			.max_extrinsic
			.unwrap();
		println!("1400 => w: {:?}", w2);
		println!("1400 => max: {:?}", max2);

		let w3 = <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(15_000_000, true);
		let max3 = <Runtime as frame_system::Config>::BlockWeights::get()
			.get(DispatchClass::Normal)
			.max_extrinsic
			.unwrap();
		println!("1500 => w: {:?}", w3);
		println!("1500 => max: {:?}", max3);

		let w4 = <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(17_000_000, true);
		let max4 = <Runtime as frame_system::Config>::BlockWeights::get()
			.get(DispatchClass::Normal)
			.max_extrinsic
			.unwrap();
		println!("1700 => w: {:?}", w4);
		println!("1700 => max: {:?}", max4);

		let w5 = <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(19_000_000, true);
		let max5 = <Runtime as frame_system::Config>::BlockWeights::get()
			.get(DispatchClass::Normal)
			.max_extrinsic
			.unwrap();
		println!("1900 => w: {:?}", w5);
		println!("1900 => max: {:?}", max5);
		assert!(false);
	}
}

// => 1200w gas
// w: Weight {   ref_time: 224886362000, proof_size: 4000000 }
// max: Weight { ref_time: 349886362000, proof_size: 3670016 }

// => 1400w gas
// w: Weight {   ref_time: 262386362000, proof_size: 4666666 }
// max: Weight { ref_time: 349886362000, proof_size: 3670016 }
