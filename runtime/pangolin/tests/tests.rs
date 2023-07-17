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

darwinia_common_runtime::impl_account_migration_tests! {}
darwinia_common_runtime::impl_evm_tests! {}
// darwinia_common_runtime::impl_fee_tests! {}
darwinia_common_runtime::impl_messages_bridge_tests! {}

mod transaction_fee {
	// darwinia
	use super::mock::*;
	// frontier
	use fp_evm::FeeCalculator;
	// substrate
	use frame_support::{dispatch::DispatchClass, pallet_prelude::Weight, traits::OnFinalize};
	use pallet_transaction_payment::Multiplier;
	use polkadot_runtime_common::{MinimumMultiplier, SlowAdjustingFeeUpdate, TargetBlockFullness};
	use sp_core::U256;
	use sp_runtime::{traits::Convert, Perbill};

	fn run_with_system_weight<F>(w: Weight, mut assertions: F)
	where
		F: FnMut() -> (),
	{
		let mut t: sp_io::TestExternalities =
			frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap().into();
		t.execute_with(|| {
			System::set_block_consumed_resources(w, 0);
			assertions()
		});
	}

	#[test]
	fn multiplier_can_grow_from_zero() {
		let minimum_multiplier = MinimumMultiplier::get();
		let target = TargetBlockFullness::get()
			* RuntimeBlockWeights::get().get(DispatchClass::Normal).max_total.unwrap();
		// if the min is too small, then this will not change, and we are doomed forever.
		// the weight is 1/100th bigger than target.
		run_with_system_weight(target.saturating_mul(101) / 100, || {
			let next = SlowAdjustingFeeUpdate::<Runtime>::convert(minimum_multiplier);
			assert!(next > minimum_multiplier, "{:?} !>= {:?}", next, minimum_multiplier);
		})
	}

	#[test]
	fn initial_evm_gas_fee_is_correct() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(TransactionPayment::next_fee_multiplier(), Multiplier::from(1u128));
			assert_eq!(
				TransactionPaymentGasPrice::min_gas_price().0,
				// U256::from(18_780_048_076_923u128)
				U256::from(16_499_762_403_421u128)
			);
		})
	}

	#[test]
	fn test_evm_fee_adjustment() {
		ExtBuilder::default().build().execute_with(|| {
			let sim = |fullness: Perbill, num_blocks: u64| -> U256 {
				let block_weight = NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT * fullness;
				for i in 0..num_blocks {
					System::set_block_number(i as u32);
					System::set_block_consumed_resources(block_weight, 0);
					TransactionPayment::on_finalize(i as u32);
				}

				TransactionPaymentGasPrice::min_gas_price().0
			};

			assert_eq!(sim(Perbill::from_percent(0), 1), U256::from(16_499_453_035_776u128),);
			assert_eq!(sim(Perbill::from_percent(25), 1), U256::from(16_499_453_035_776u128),);
			assert_eq!(sim(Perbill::from_percent(50), 1), U256::from(16_499_762_403_421u128),);
			assert_eq!(sim(Perbill::from_percent(100), 1), U256::from(165_00_690_541_159u128),);

			// 1 "real" hour (at 12-second blocks)
			// println!("{}", sim(Perbill::from_percent(0), 300));
			// println!("{}", sim(Perbill::from_percent(25), 300));
			// println!("{}", sim(Perbill::from_percent(50), 300));
			// println!("{}", sim(Perbill::from_percent(100), 300));
			assert_eq!(sim(Perbill::from_percent(0), 300), U256::from(16_408_134_714_177u128));
			assert_eq!(sim(Perbill::from_percent(25), 300), U256::from(16_408_134_714_177u128),);
			assert_eq!(sim(Perbill::from_percent(50), 300), U256::from(16_500_690_541_159u128),);
			assert_eq!(sim(Perbill::from_percent(100), 300), U256::from(16_781_502_380_018u128),);

			// 1 "real" day (at 12-second blocks)
			// println!("{}", sim(Perbill::from_percent(0), 7200));
			// println!("{}", sim(Perbill::from_percent(25), 7200));
			// println!("{}", sim(Perbill::from_percent(50), 7200));
			// println!("{}", sim(Perbill::from_percent(100), 7200));
			assert_eq!(sim(Perbill::from_percent(0), 7200), U256::from(14_662_265_651_569u128),);
			assert_eq!(sim(Perbill::from_percent(25), 7200), U256::from(14_662_265_651_569u128),);
			assert_eq!(sim(Perbill::from_percent(50), 7200), U256::from(16_781_502_380_018u128));
			assert_eq!(sim(Perbill::from_percent(100), 7200), U256::from(25_160_548_467_697u128),);

			// 7 "real" day (at 12-second blocks)
			// println!("{}", sim(Perbill::from_percent(0), 50400));
			// println!("{}", sim(Perbill::from_percent(25), 50400));
			// println!("{}", sim(Perbill::from_percent(50), 50400));
			// println!("{}", sim(Perbill::from_percent(100), 50400));
			assert_eq!(sim(Perbill::from_percent(0), 50400), U256::from(9_779_391_182_619u128),);
			assert_eq!(sim(Perbill::from_percent(25), 50400), U256::from(9_779_391_182_619u128),);
			assert_eq!(sim(Perbill::from_percent(50), 50400), U256::from(25_160_548_467_697u128));
			assert_eq!(sim(Perbill::from_percent(100), 50400), U256::from(428_494_211_541_821u128),);
		})
	}
}
