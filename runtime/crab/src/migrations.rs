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

	let removed_items: &[(&[u8], &[&[u8]])] = &[
		(b"FromDarwiniaIssuing", &[b"MappingFactoryAddress", b"RemoteBackingAccount"]),
		(b"KtonTreasury", &[b"ProposalCount", b"Proposals", b"Approvals"]),
		(
			b"ToCrabParachainBacking",
			&[b"SecureLimitedPeriod", b"TransactionInfos", b"RemoteMappingTokenFactoryAccount"],
		),
	];
	let hash = &[];

	removed_items.iter().for_each(|(module, items)| {
		items.iter().for_each(|item| migration::remove_storage_prefix(module, item, hash));
	});

	// 5EYCAe5gKAhKXbKVquxUAg1Z22qvbkp8Ddmrmp5pCbKRHcs8
	// 0x6d6f646c64612f73327362610000000000000000000000000000000000000000
	let from = AccountId::from([
		109, 111, 100, 108, 100, 97, 47, 115, 50, 115, 98, 97, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0,
	]);
	// 5ELRpquT7C3mWtjeomFr5wb6ZkenTLEDNH6yFmKMudqyWLo4
	// 0x64766d3a000000000000002401224012bae7c2f217392665ca7abc16dcde1e16
	let to = AccountId::from([
		100, 118, 109, 58, 0, 0, 0, 0, 0, 0, 0, 36, 1, 34, 64, 18, 186, 231, 194, 242, 23, 57, 38,
		101, 202, 122, 188, 22, 220, 222, 30, 22,
	]);
	let _ = Balances::transfer_all(Origin::signed(from.clone()), Address::from(to.clone()), false);

	// 0
	RuntimeBlockWeights::get().max_block
}
