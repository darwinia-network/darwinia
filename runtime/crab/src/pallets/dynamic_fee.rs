// --- paritytech ---
use sp_core::U256;
// --- darwinia-network ---
use crate::*;
use dvm_dynamic_fee::Config;

frame_support::parameter_types! {
	pub BoundDivision: U256 = U256::from(1024);
}

impl Config for Runtime {
	type Event = Event;
	type MinGasPriceBoundDivisor = BoundDivision;
}
