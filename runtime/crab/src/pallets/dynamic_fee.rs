// --- paritytech ---
use sp_core::U256;
// --- darwinia-network ---
use crate::*;
use dvm_dynamic_fee::Config;

frame_support::parameter_types! {
	pub BoundDivision: U256 = 1024.into();
}

impl Config for Runtime {
	type MinGasPriceBoundDivisor = BoundDivision;
}
