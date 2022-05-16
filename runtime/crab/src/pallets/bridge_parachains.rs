pub use pallet_bridge_parachains::Instance1 as WithKusamaParachainsInstance;


// --- darwinia-network ---
use crate::*;
use pallet_bridge_parachains::Config;

impl Config<WithKusamaParachainsInstance> for Runtime {
    type BridgesGrandpaPalletInstance = WithKusamaGrandpa;
    type HeadsToKeep = HeadersToKeep;
    type ParasPalletName = bp_kusama::PARAS_PALLET_NAME;
}