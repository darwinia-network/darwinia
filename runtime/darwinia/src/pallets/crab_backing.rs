// --- darwinia-network ---
use crate::*;
use darwinia_crab_backing::Config;

frame_support::parameter_types! {
	pub const CrabBackingPalletId: PalletId = PalletId(*b"da/crabk");
}

impl Config for Runtime {
	type PalletId = CrabBackingPalletId;
	type RingCurrency = Ring;
	type WeightInfo = ();
}
