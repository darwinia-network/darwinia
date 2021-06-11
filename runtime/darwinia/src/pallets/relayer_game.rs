pub use darwinia_relayer_game::Instance1 as EthereumRelayerGameInstance;

// --- substrate ---
use frame_support::traits::LockIdentifier;
// --- darwinia ---
use crate::*;
use darwinia_relayer_game::Config;

frame_support::parameter_types! {
	pub const EthereumRelayerGameLockId: LockIdentifier = *b"da/rgame";
}

impl Config<EthereumRelayerGameInstance> for Runtime {
	type RingCurrency = Ring;
	type LockId = EthereumRelayerGameLockId;
	type RingSlash = Treasury;
	type RelayerGameAdjustor = EthereumRelayerGameAdjustor;
	type RelayableChain = EthereumRelay;
	type WeightInfo = ();
}
