// darwinia
use crate::gov_origin::ROOT;
use dc_primitives::{AccountId, Balance};
// substrate
use frame_support::{traits::LockIdentifier, PalletId};
use sp_std::prelude::*;

#[cfg(not(feature = "runtime-benchmarks"))]
pub const EXISTENTIAL_DEPOSIT: Balance = 0;
#[cfg(feature = "runtime-benchmarks")]
pub const EXISTENTIAL_DEPOSIT: Balance = 1;

frame_support::parameter_types! {
	pub const TreasuryPid: PalletId = PalletId(*b"da/trsry");

	pub const FeeMarketLid: LockIdentifier = *b"da/feecr";

	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;

	pub  AssetCreators: Vec<AccountId> = vec![ROOT];
}
