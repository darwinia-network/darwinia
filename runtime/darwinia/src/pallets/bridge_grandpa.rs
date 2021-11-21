pub use pallet_bridge_grandpa::Instance1 as WithCrabGrandpa;

// --- paritytech ---
use pallet_bridge_grandpa::{weights::RialtoWeight, Config};
// --- darwinia-network ---
use crate::*;
use bridge_primitives::Crab;

frame_support::parameter_types! {
	// This is a pretty unscientific cap.
	//
	// Note that once this is hit the pallet will essentially throttle incoming requests down to one
	// call per block.
	pub const MaxRequests: u32 = 50;
	// Number of headers to keep.
	//
	// Assuming the worst case of every header being finalized, we will keep headers for at least a
	// week.
	pub const HeadersToKeep: u32 = 7 * DAYS as u32;
}

impl Config<WithCrabGrandpa> for Runtime {
	type BridgedChain = Crab;
	type MaxRequests = MaxRequests;
	type HeadersToKeep = HeadersToKeep;
	// FIXME
	type WeightInfo = RialtoWeight<Runtime>;
}
