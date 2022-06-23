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
	use fp_storage::{EthereumStorageSchema, PALLET_ETHEREUM_SCHEMA};

	frame_support::storage::unhashed::put::<EthereumStorageSchema>(
		&PALLET_ETHEREUM_SCHEMA,
		&EthereumStorageSchema::V3,
	);

	// 0
	RuntimeBlockWeights::get().max_block
}
