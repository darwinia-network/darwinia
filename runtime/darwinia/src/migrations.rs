// --- paritytech ---
#[allow(unused)]
use frame_support::{migration, traits::OnRuntimeUpgrade, weights::Weight};
// --- darwinia-network ---
#[allow(unused)]
use crate::*;

pub struct CustomOnRuntimeUpgrade;
impl OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		Ok(())
	}

	fn on_runtime_upgrade() -> Weight {
		migrate()
	}
}

fn migrate() -> Weight {
	migration::move_pallet(b"DarwiniaHeaderMMR", b"DarwiniaHeaderMmr");
	migration::move_pallet(b"Instance1DarwiniaRelayAuthorities", b"EcdsaRelayAuthority");

	// Backing AccountId: 2qeMxq616BhswXHiiHp7H4VgaVv2S8xwkzWkoyoxcTA8v1YA
	let from = AccountId::from([
		109, 111, 100, 108, 100, 97, 47, 116, 99, 114, 98, 107, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0,
	]);
	// HelixDaoMultisig AccountId:
	// 0xBd1a110ec476b4775c43905000288881367B1a88(2qSbd2umtD4KmV2X89YfqmCQgDraEabAaLNFiR96xUJ1m31G)
	let to = AccountId::from([
		100, 118, 109, 58, 0, 0, 0, 0, 0, 0, 0, 189, 26, 17, 14, 196, 118, 180, 119, 92, 67, 144,
		80, 0, 40, 136, 129, 54, 123, 26, 136, 173,
	]);
	let _ = Ring::transfer_all(Origin::signed(from.clone()), Address::from(to.clone()), false);

	// 0
	RuntimeBlockWeights::get().max_block
}
