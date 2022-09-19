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
	let authorities = [
		"0x953d65e6054b7eb1629f996238c0aa9b4e2dbfe9",
		"0x7c9b3d4cfc78c681b7460acde2801452aef073a9",
		"0x717c38fd5fdecb1b105a470f861b33a6b0f9f7b8",
		"0x3e25247CfF03F99a7D83b28F207112234feE73a6",
		"0x2EaBE5C6818731E282B80De1a03f8190426e0Dd9",
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

	// 0
	RuntimeBlockWeights::get().max_block
}
