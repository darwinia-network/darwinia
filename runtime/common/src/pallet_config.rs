// darwinia
use dc_primitives::*;
// substrate
use sp_runtime::traits::AccountIdConversion;
use sp_std::prelude::*;

#[cfg(not(feature = "runtime-benchmarks"))]
pub const EXISTENTIAL_DEPOSIT: Balance = 0;
#[cfg(feature = "runtime-benchmarks")]
pub const EXISTENTIAL_DEPOSIT: Balance = 1;

frame_support::parameter_types! {
	pub const TreasuryPid: frame_support::PalletId = frame_support::PalletId(*b"da/trsry");

	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
	pub const MaxBalance: Balance = Balance::max_value();

	pub const RelayOrigin: cumulus_primitives_core::AggregateMessageOrigin = cumulus_primitives_core::AggregateMessageOrigin::Parent;

	pub  AssetCreators: Vec<AccountId> = vec![super::gov_origin::ROOT];
	pub TreasuryAccount: AccountId = TreasuryPid::get().into_account_truncating();
}
