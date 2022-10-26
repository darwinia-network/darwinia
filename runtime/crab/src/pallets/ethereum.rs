// --- darwinia-network ---
use crate::*;
use darwinia_ethereum::{Config, IntermediateStateRoot};

impl Config for Runtime {
	type Event = Event;
	type StateRoot = IntermediateStateRoot;
}
