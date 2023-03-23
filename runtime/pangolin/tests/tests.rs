pub mod mock;

darwinia_common_runtime::impl_account_migration_tests! {}

darwinia_common_runtime::impl_evm_tests! {}

#[cfg(test)]
mod transaction_fee_tests {
	// darwinia
	use crate::mock::*;
	// substrate
	use fp_evm::FeeCalculator;
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
	fn initial_gas_fee_is_correct() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(TransactionPayment::next_fee_multiplier(), Multiplier::from(1u128));
			assert_eq!(DynamicGasPrice::min_gas_price().0, U256::from(1_878_004_808u128));
		})
	}

	#[test]
	fn test_fee_scenarios() {
		use sp_runtime::FixedU128;
		ExtBuilder::default().build().execute_with(|| {
			let sim = |fullness: Perbill, num_blocks: u64| -> U256 {
				let block_weight = NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT * fullness;
				for i in 0..num_blocks {
					System::set_block_number(i as u32);
					System::set_block_consumed_resources(block_weight, 0);
					TransactionPayment::on_finalize(i as u32);
				}

				DynamicGasPrice::min_gas_price().0
			};

			assert_eq!(sim(Perbill::from_percent(0), 1), U256::from(1_877_969_595),);
			assert_eq!(sim(Perbill::from_percent(25), 1), U256::from(1_877_969_595),);
			assert_eq!(sim(Perbill::from_percent(50), 1), U256::from(1_878_004_808),);
			assert_eq!(sim(Perbill::from_percent(100), 1), U256::from(1_878_110_448),);

			// 1 "real" hour (at 12-second blocks)
			assert_eq!(sim(Perbill::from_percent(0), 300), U256::from(1_867_575_734));
			assert_eq!(sim(Perbill::from_percent(25), 300), U256::from(1_867_575_734),);
			assert_eq!(sim(Perbill::from_percent(50), 300), U256::from(1_878_110_448),);
			assert_eq!(sim(Perbill::from_percent(100), 300), U256::from(1_910_072_483),);

			// 1 "real" day (at 12-second blocks)
			assert_eq!(sim(Perbill::from_percent(0), 7200), U256::from(1_668_860_721u128),);
			assert_eq!(sim(Perbill::from_percent(25), 7200), U256::from(1_668_860_721u128),);
			assert_eq!(sim(Perbill::from_percent(50), 7200), U256::from(1_910_072_483u128));
			assert_eq!(sim(Perbill::from_percent(100), 7200), U256::from(2_863_776_449u128),);

			// 7 "real" day (at 12-second blocks)
			assert_eq!(sim(Perbill::from_percent(0), 50400), U256::from(1_113_091_401u128),);
			assert_eq!(sim(Perbill::from_percent(25), 50400), U256::from(1_113_091_401u128),);
			assert_eq!(sim(Perbill::from_percent(50), 50400), U256::from(2_863_776_449u128));
			assert_eq!(sim(Perbill::from_percent(100), 50400), U256::from(48_771_259_233u128),);
		})
	}
}
