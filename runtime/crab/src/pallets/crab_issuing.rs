// --- substrate ---
use sp_runtime::ModuleId;
// --- darwinia ---
use crate::*;
use darwinia_crab_issuing::Config;

frame_support::parameter_types! {
	pub const CrabIssuingModuleId: ModuleId = ModuleId(*b"da/crais");
}
impl Config for Runtime {
	type Event = Event;
	type ModuleId = CrabIssuingModuleId;
	type RingCurrency = Ring;
	type WeightInfo = ();
}
