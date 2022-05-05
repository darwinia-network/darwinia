// --- paritytech ---
use pallet_sudo::Config;
// --- darwinia-network ---
use crate::*;

impl Config for Runtime {
	type Call = Call;
	type Event = Event;
}
