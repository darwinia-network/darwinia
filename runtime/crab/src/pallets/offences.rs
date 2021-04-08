// --- substrate ---
use frame_support::weights::Weight;
use pallet_offences::Config;
use pallet_session::historical::IdentificationTuple;
use sp_runtime::Perbill;
// --- darwinia ---
use crate::*;

frame_support::parameter_types! {
	pub OffencesWeightSoftLimit: Weight = Perbill::from_percent(60)
		* BlockWeights::get().max_block;
}
impl Config for Runtime {
	type Event = Event;
	type IdentificationTuple = IdentificationTuple<Self>;
	type OnOffenceHandler = Staking;
	type WeightSoftLimit = OffencesWeightSoftLimit;
}
