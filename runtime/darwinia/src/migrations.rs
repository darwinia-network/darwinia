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

	const REVERT_BYTECODE: [u8; 5] = [0x60, 0x00, 0x60, 0x00, 0xFD];
	for precompile in DarwiniaPrecompiles::<Runtime>::used_addresses() {
		EVM::create_account(&precompile, REVERT_BYTECODE.to_vec());
	}

	// 0
	RuntimeBlockWeights::get().max_block
}
