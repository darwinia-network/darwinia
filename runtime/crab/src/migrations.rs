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
		Scheduler::pre_migrate_to_v3()?;

		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		Scheduler::post_migrate_to_v3()?;

		Ok(())
	}

	fn on_runtime_upgrade() -> Weight {
		migrate()
	}
}

fn migrate() -> Weight {
	Scheduler::migrate_v2_to_v3();

	for precompile in CrabPrecompiles::<Runtime>::used_addresses() {
		EVM::create_account(&precompile, vec![0x60, 0x00, 0x60, 0x00, 0xFD]);
	}

	let module = b"FromDarwiniaIssuing";
	migration::remove_storage_prefix(module, b"MappingFactoryAddress", &[]);
	migration::remove_storage_prefix(module, b"RemoteBackingAccount", &[]);

	// TODO
	// Do we need a migration for KtonTreasury?

	// 0
	RuntimeBlockWeights::get().max_block
}
