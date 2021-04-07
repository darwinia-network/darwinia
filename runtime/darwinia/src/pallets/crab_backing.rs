// --- darwinia ---
use crate::*;
use darwinia_crab_backing::Config;

frame_support::parameter_types! {
	pub const CrabBackingModuleId: ModuleId = ModuleId(*b"da/crabk");
}
impl Config for Runtime {
	type ModuleId = CrabBackingModuleId;
	type RingCurrency = Ring;
	type WeightInfo = ();
}
