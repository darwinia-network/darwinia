// --- darwinia ---
use crate::*;
use darwinia_tron_backing::Config;

frame_support::parameter_types! {
	pub const TronBackingModuleId: ModuleId = ModuleId(*b"da/trobk");
}
impl Config for Runtime {
	type ModuleId = TronBackingModuleId;
	type RingCurrency = Ring;
	type KtonCurrency = Kton;
	type WeightInfo = ();
}
