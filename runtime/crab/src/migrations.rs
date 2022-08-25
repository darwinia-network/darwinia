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

	// WKTON: 0x159933C635570D5042723359fbD1619dFe83D3f3(5ELRpquT7C3mWtjeoTzuGDz3Df1pPt7UekEogDKMcMViEZFF)
	let from = AccountId::from([
		100, 118, 109, 58, 0, 0, 0, 0, 0, 0, 0, 21, 153, 51, 198, 53, 87, 13, 80, 66, 114, 51, 89,
		251, 209, 97, 157, 254, 131, 211, 243, 210,
	]);
	// Migration:
	// 0xD5f4940704Eb4cE5e4b51877d49B58A3e93531b6(5ELRpquT7C3mWtjesSS6XvZDxwT8PMpDGJNaDGWLH3rFEsgf)
	let to = AccountId::from([
		100, 118, 109, 58, 0, 0, 0, 0, 0, 0, 0, 213, 244, 148, 7, 4, 235, 76, 229, 228, 181, 24,
		119, 212, 155, 88, 163, 233, 53, 49, 182, 96,
	]);

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
