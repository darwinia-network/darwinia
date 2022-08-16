// --- paritytech ---
use pallet_utility::Config;
// --- darwinia-network ---
use crate::*;

impl Config for Runtime {
	type Call = Call;
	type Event = Event;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}
