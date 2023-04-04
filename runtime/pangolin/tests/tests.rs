pub mod mock;

darwinia_common_runtime::impl_account_migration_tests! {}

darwinia_common_runtime::impl_evm_tests! {}

darwinia_common_runtime::impl_fee_tests! {}

#[cfg(test)]
mod messsages_tests {
	use super::mock::*;
	use static_assertions::assert_type_eq_all;

	#[test]
	fn darwinia_const_variables_should_match() {
		assert_eq!(bp_darwinia_core::MILLISECS_PER_BLOCK, dc_primitives::MILLISECS_PER_BLOCK);
		assert_eq!(bp_darwinia_core::MINUTES, dc_primitives::MINUTES);
		assert_eq!(bp_darwinia_core::HOURS, dc_primitives::HOURS);
		assert_eq!(bp_darwinia_core::DAYS, dc_primitives::DAYS);

		assert_eq!(bp_darwinia_core::AVERAGE_ON_INITIALIZE_RATIO, AVERAGE_ON_INITIALIZE_RATIO);
		assert_eq!(bp_darwinia_core::NORMAL_DISPATCH_RATIO, NORMAL_DISPATCH_RATIO);
		assert_eq!(bp_darwinia_core::WEIGHT_MILLISECS_PER_BLOCK, WEIGHT_MILLISECS_PER_BLOCK);
		assert_eq!(bp_darwinia_core::MAXIMUM_BLOCK_WEIGHT, MAXIMUM_BLOCK_WEIGHT);

		assert_eq!(bp_darwinia_core::RuntimeBlockLength::get().max, RuntimeBlockLength::get().max);
	}

	#[test]
	fn darwinia_basic_type_should_match() {
		assert_type_eq_all!(bp_darwinia_core::BlockNumber, u32);
		assert_type_eq_all!(bp_darwinia_core::Hash, sp_core::H256);
		assert_type_eq_all!(bp_darwinia_core::Nonce, u32);
		assert_type_eq_all!(bp_darwinia_core::Balance, u128);
		assert_type_eq_all!(bp_darwinia_core::AccountId, AccountId);
	}

	#[test]
	fn polkadot_const_variables_should_match() {
		assert_eq!(
			bp_polkadot_core::NORMAL_DISPATCH_RATIO,
			polkadot_runtime_common::NORMAL_DISPATCH_RATIO
		);
		assert_eq!(
			bp_polkadot_core::MAXIMUM_BLOCK_WEIGHT,
			polkadot_runtime_common::MAXIMUM_BLOCK_WEIGHT
		);
		assert_eq!(
			bp_polkadot_core::AVERAGE_ON_INITIALIZE_RATIO,
			polkadot_runtime_common::AVERAGE_ON_INITIALIZE_RATIO
		);
		assert_eq!(
			bp_polkadot_core::BlockLength::get().max,
			polkadot_runtime_common::BlockLength::get().max
		);
	}

	#[test]
	fn polkadot_basic_type_should_match() {
		assert_type_eq_all!(bp_polkadot_core::BlockNumber, polkadot_primitives::BlockNumber);
		assert_type_eq_all!(bp_polkadot_core::Balance, polkadot_primitives::Balance);
		assert_type_eq_all!(bp_polkadot_core::Hash, polkadot_primitives::Hash);
		assert_type_eq_all!(bp_polkadot_core::Index, polkadot_primitives::AccountIndex);
		assert_type_eq_all!(bp_polkadot_core::Nonce, polkadot_primitives::Nonce);
		assert_type_eq_all!(bp_polkadot_core::Signature, polkadot_primitives::Signature);
		assert_type_eq_all!(bp_polkadot_core::AccountId, polkadot_primitives::AccountId);
		assert_type_eq_all!(bp_polkadot_core::Header, polkadot_primitives::Header);
	}

	#[test]
	fn block_execution_and_extrinsic_base_weight_should_match() {
		assert_eq!(
			weights::BlockExecutionWeight::get(),
			frame_support::weights::constants::BlockExecutionWeight::get(),
		);
		assert_eq!(
			weights::ExtrinsicBaseWeight::get(),
			frame_support::weights::constants::ExtrinsicBaseWeight::get(),
		);
	}
}
