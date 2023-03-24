pub mod mock;

#[cfg(test)]
darwinia_common_runtime::impl_account_migration_tests! {}

#[cfg(test)]
darwinia_common_runtime::impl_evm_tests! {}

#[cfg(test)]
darwinia_common_runtime::impl_fee_tests! {}
