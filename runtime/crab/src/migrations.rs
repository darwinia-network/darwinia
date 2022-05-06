// --- paritytech ---
#[allow(unused)]
use frame_support::{migration, traits::OnRuntimeUpgrade, weights::Weight};
use frame_support::{traits::Currency, PalletId};
use sp_runtime::traits::AccountIdConversion;
// --- darwinia-network ---
#[allow(unused)]
use crate::*;
use darwinia_support::traits::LockableCurrency;

fn migrate() -> Weight {
	migration::remove_storage_prefix(b"DarwiniaClaims", b"ClaimsFromEth", &[]);
	migration::remove_storage_prefix(b"DarwiniaClaims", b"ClaimsFromTron", &[]);

	let claims_pallet_id = PalletId(*b"da/claim");
	let claims_pallet_account = claims_pallet_id.into_account();
	let treasury_account = PalletId(*b"da/trsry").into_account();

	// We mint this ED before. Clean it.
	Ring::remove_lock(claims_pallet_id.0, &claims_pallet_account);
	let _ = Ring::slash(&claims_pallet_account, 1 * COIN);
	// Transfer all balances to treasury account.
	let _ = Ring::transfer_all(Origin::signed(claims_pallet_account), treasury_account, false);

	// 0
	RuntimeBlockWeights::get().max_block
}

pub struct CustomOnRuntimeUpgrade;
impl OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		let claims_pallet_account = PalletId(*b"da/claim").into_account();

		assert!(Ring::free_balance(&claims_pallet_account) != 0);

		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		let claims_pallet_account = PalletId(*b"da/claim").into_account();

		assert!(Ring::free_balance(&claims_pallet_account) == 0);

		Ok(())
	}

	fn on_runtime_upgrade() -> Weight {
		migrate()
	}
}
