// --- paritytech ---
use pallet_authority_discovery::Config;
// --- darwinia-network ---
use crate::*;

frame_support::parameter_types! {
	pub const MaxAuthorities: u32 = 100_000;
}

impl Config for Runtime {
	type MaxAuthorities = MaxAuthorities;
}
