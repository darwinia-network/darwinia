// --- paritytech ---
use frame_support::PalletId;
// --- darwinia-network ---
use crate::*;
use darwinia_crab_issuing::Config;

frame_support::parameter_types! {
	pub const CrabIssuingPalletId: PalletId = PalletId(*b"da/crais");
}

impl Config for Runtime {
	type PalletId = CrabIssuingPalletId;
	type RingCurrency = Ring;
	type WeightInfo = ();
}
