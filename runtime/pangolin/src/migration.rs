// darwinia
use crate::*;

pub struct CustomOnRuntimeUpgrade;
impl frame_support::traits::OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
		Ok(())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		migrate()
	}
}

fn migrate() -> frame_support::weights::Weight {
	<pallet_assets::migration::v1::MigrateToV1<Runtime> as frame_support::traits::OnRuntimeUpgrade>::on_runtime_upgrade();
	frame_support::migration::move_pallet(b"Staking", b"DarwiniaStaking");

	// frame_support::weights::Weight::zero()
	RuntimeBlockWeights::get().max_block
}
