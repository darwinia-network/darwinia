pub mod mock;

darwinia_common_runtime::impl_account_migration_tests! {}

darwinia_common_runtime::impl_evm_tests! {}

darwinia_common_runtime::impl_fee_tests! {}

#[cfg(test)]
mod messsages_tests {
	use std::sync::Arc;

	use sp_core::TypedGet;

	use super::mock::*;
	use static_assertions::assert_type_eq_all;

	#[test]
	fn const_variables_should_match() {
		assert_eq!(bp_darwinia_core::MILLISECS_PER_BLOCK, dc_primitives::MILLISECS_PER_BLOCK);
		assert_eq!(bp_darwinia_core::MINUTES, dc_primitives::MINUTES);
		assert_eq!(bp_darwinia_core::HOURS, dc_primitives::HOURS);
		assert_eq!(bp_darwinia_core::DAYS, dc_primitives::DAYS);

		assert_eq!(bp_darwinia_core::AVERAGE_ON_INITIALIZE_RATIO, AVERAGE_ON_INITIALIZE_RATIO);
		assert_eq!(bp_darwinia_core::NORMAL_DISPATCH_RATIO, NORMAL_DISPATCH_RATIO);
		assert_eq!(bp_darwinia_core::WEIGHT_MILLISECS_PER_BLOCK, WEIGHT_MILLISECS_PER_BLOCK);
		assert_eq!(bp_darwinia_core::MAXIMUM_BLOCK_WEIGHT, MAXIMUM_BLOCK_WEIGHT);
		assert_eq!(
			bp_darwinia_core::RuntimeBlockWeights::get().base_block,
			RuntimeBlockWeights::get().base_block
		);
		assert_eq!(
			bp_darwinia_core::RuntimeBlockWeights::get().max_block,
			RuntimeBlockWeights::get().max_block
		);
		use frame_support::dispatch::DispatchClass;
		assert_eq!(
			bp_darwinia_core::RuntimeBlockWeights::get().get(DispatchClass::Normal).max_extrinsic,
			RuntimeBlockWeights::get().get(DispatchClass::Normal).max_extrinsic
		);
		assert_eq!(bp_darwinia_core::RuntimeBlockLength::get().max, RuntimeBlockLength::get().max);
	}

	#[test]
	fn basic_type_should_match() {
		assert_type_eq_all!(bp_darwinia_core::BlockNumber, u32);
		assert_type_eq_all!(bp_darwinia_core::Hash, sp_core::H256);
		assert_type_eq_all!(bp_darwinia_core::Nonce, u32);
		assert_type_eq_all!(bp_darwinia_core::Balance, u128);
		assert_type_eq_all!(bp_darwinia_core::AccountId, AccountId);
	}
}
