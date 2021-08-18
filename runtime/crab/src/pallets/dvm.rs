// --- darwinia-network ---
use crate::*;
use dvm_ethereum::{Config, IntermediateStateRoot};

impl Config for Runtime {
	type Event = Event;
	type StateRoot = IntermediateStateRoot;
	type RingCurrency = Ring;
	type KtonCurrency = Kton;
}
