// --- darwinia-network ---
use crate::*;
use dvm_ethereum::{Config, IntermediateStateRoot};
// --- paritytech ---
use frame_support::PalletId;

frame_support::parameter_types! {
	pub const DvmPalletId: PalletId = PalletId(*b"dar/dvmp");
}

impl Config for Runtime {
	type PalletId = DvmPalletId;
	type Event = Event;
	type StateRoot = IntermediateStateRoot;
	type RingCurrency = Ring;
	type KtonCurrency = Kton;
}
