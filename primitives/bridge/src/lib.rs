#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

//! Darwinia bridge types shared between the runtime and the node-side code.

// --- paritytech ---
use sp_core::H256;
use sp_runtime::traits::Convert;
// --- darwinia-network ---
use common_primitives::AccountId;

/// Convert a 256-bit hash into an AccountId.
pub struct AccountIdConverter;
impl Convert<H256, AccountId> for AccountIdConverter {
	fn convert(hash: H256) -> AccountId {
		hash.to_fixed_bytes().into()
	}
}
