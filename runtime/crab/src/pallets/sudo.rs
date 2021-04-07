// --- substrate ---
use pallet_sudo::Config;
// --- darwinia ---
use crate::*;

impl Config for Runtime {
	type Event = Event;
	type Call = Call;
}
