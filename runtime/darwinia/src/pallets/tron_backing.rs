// --- paritytech ---
use frame_support::PalletId;
// --- darwinia-network ---
use crate::*;
use to_tron_backing::Config;

frame_support::parameter_types! {
	pub const TronBackingPalletId: PalletId = PalletId(*b"da/trobk");
}

impl Config for Runtime {
	type KtonCurrency = Kton;
	type PalletId = TronBackingPalletId;
	type RingCurrency = Ring;
	type WeightInfo = ();
}
