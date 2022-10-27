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

	for precompile in DarwiniaPrecompiles::<Runtime>::used_addresses() {
		EVM::create_account(&precompile, vec![0x60, 0x00, 0x60, 0x00, 0xFD]);
	}

	let removed_items: &[(&[u8], &[&[u8]])] = &[
		(
			b"ToCrabBacking",
			&[
				b"SecureLimitedPeriod",
				b"SecureLimitedRingAmount",
				b"TransactionInfos",
				b"RemoteMappingTokenFactoryAccount",
			],
		),
		(b"KtonTreasury", &[b"ProposalCount", b"Proposals", b"Approvals"]),
	];
	let hash = &[];

	removed_items.iter().for_each(|(module, items)| {
		items.iter().for_each(|item| migration::remove_storage_prefix(module, item, hash));
	});

	// 0
	RuntimeBlockWeights::get().max_block
}
