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
	use frame_support::StorageHasher;

	// TODO
	let from = AccountId::from([]);
	let to = AccountId::from([]);

	let _ = Kton::transfer_all(Origin::signed(from.clone()), Address::from(to.clone()), false);
	if let Some(v) = migration::take_storage_value::<Balance>(
		b"Ethereum",
		b"RemainingKtonBalance",
		&frame_support::Blake2_128Concat::hash(from.as_ref()),
	) {
		migration::put_storage_value(
			b"Ethereum",
			b"RemainingKtonBalance",
			&frame_support::Blake2_128Concat::hash(to.as_ref()),
			v,
		);
	}

	migration::move_pallet(b"DarwiniaHeaderMMR", b"DarwiniaHeaderMmr");

	// 0
	RuntimeBlockWeights::get().max_block
}
