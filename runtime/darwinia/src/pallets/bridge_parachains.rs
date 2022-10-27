pub use pallet_bridge_parachains::Instance1 as WithPolkadotParachainsInstance;

// --- darwinia-network ---
use crate::*;
use pallet_bridge_parachains::Config;

impl Config<WithPolkadotParachainsInstance> for Runtime {
	type BridgesGrandpaPalletInstance = WithPolkadotGrandpa;
	type HeadsToKeep = PolkadotHeadersToKeep;
}
