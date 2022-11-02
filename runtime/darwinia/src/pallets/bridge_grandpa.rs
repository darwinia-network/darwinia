pub use pallet_bridge_grandpa::{Instance1 as WithCrabGrandpa, Instance2 as WithPolkadotGrandpa};

// --- darwinia-network ---
use crate::*;
use pallet_bridge_grandpa::Config;

frame_support::parameter_types! {
	// This is a pretty unscientific cap.
	//
	// Note that once this is hit the pallet will essentially throttle incoming requests down to one
	// call per block.
	pub const MaxRequests: u32 = 50;
	pub const CrabHeadersToKeep: u32 = 6_000;
	pub const PolkadotHeadersToKeep: u32 = 500;
}

impl Config<WithCrabGrandpa> for Runtime {
	type BridgedChain = bp_crab::Crab;
	type HeadersToKeep = CrabHeadersToKeep;
	type MaxRequests = MaxRequests;
	type WeightInfo = ();
}
impl Config<WithPolkadotGrandpa> for Runtime {
	type BridgedChain = bp_polkadot::Polkadot;
	type HeadersToKeep = PolkadotHeadersToKeep;
	type MaxRequests = MaxRequests;
	type WeightInfo = ();
}
