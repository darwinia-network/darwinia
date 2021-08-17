// --- paritytech ---
use pallet_utility::Config;
// --- darwinia-network ---
use crate::{weights::pallet_utility::WeightInfo, *};

impl Config for Runtime {
	type Event = Event;
	type Call = Call;
	type WeightInfo = WeightInfo<Runtime>;
}
