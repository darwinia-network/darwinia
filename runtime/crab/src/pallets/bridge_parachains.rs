pub use pallet_bridge_parachains::Instance1 as WithKusamaParachainsInstance;

// --- darwinia-network ---
use crate::*;
use pallet_bridge_parachains::Config;

frame_support::parameter_types! {
	pub const KusamaParasPalletName: &'static str = bp_kusama::PARAS_PALLET_NAME;
}

impl Config<WithKusamaParachainsInstance> for Runtime {
	type BridgesGrandpaPalletInstance = WithKusamaGrandpa;
	type HeadsToKeep = KusamaHeadersToKeep;
	type ParasPalletName = KusamaParasPalletName;
}
