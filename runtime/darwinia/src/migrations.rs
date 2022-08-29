#[allow(unused)]
use {
	crate::*,
	frame_support::{migration, traits::OnRuntimeUpgrade, weights::Weight},
	sp_std::prelude::*,
};

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

	let authorities = [
		"0x953d65e6054b7eb1629f996238c0aa9b4e2dbfe9",
		"0x7c9b3d4cfc78c681b7460acde2801452aef073a9",
		"0x717c38fd5fdecb1b105a470f861b33a6b0f9f7b8",
		"0x3e25247CfF03F99a7D83b28F207112234feE73a6",
	]
	.iter()
	.filter_map(|s| array_bytes::hex_into(s).ok())
	.collect::<Vec<_>>();

	if !authorities.is_empty() {
		if let Ok(authorities) = frame_support::BoundedVec::try_from(authorities) {
			<darwinia_ecdsa_authority::Authorities<Runtime>>::put(authorities.clone());
			<darwinia_ecdsa_authority::NextAuthorities<Runtime>>::put(authorities);
		}
	}

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
