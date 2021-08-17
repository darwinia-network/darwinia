// --- paritytech ---
use pallet_offences::Config;
use pallet_session::historical::IdentificationTuple;
// --- darwinia-network ---
use crate::*;

impl Config for Runtime {
	type Event = Event;
	type IdentificationTuple = IdentificationTuple<Self>;
	type OnOffenceHandler = Staking;
}
