#[cfg(feature = "std")]
pub use darwinia_ethereum_relay::DagsMerkleRootsLoader;

// --- substrate ---
use sp_runtime::{ModuleId, Perbill};
// --- darwinia ---
use crate::*;
use darwinia_ethereum_relay::Config;
use ethereum_primitives::EthereumNetworkType;

frame_support::parameter_types! {
	pub const EthereumRelayModuleId: ModuleId = ModuleId(*b"da/ethrl");
	pub const EthereumNetwork: EthereumNetworkType = EthereumNetworkType::Ropsten;
	pub const ConfirmPeriod: BlockNumber = 200;
	pub const ApproveThreshold: Perbill = Perbill::from_percent(60);
	pub const RejectThreshold: Perbill = Perbill::from_percent(1);
}
impl Config for Runtime {
	type ModuleId = EthereumRelayModuleId;
	type Event = Event;
	type EthereumNetwork = EthereumNetwork;
	type Call = Call;
	type Currency = Ring;
	type RelayerGame = EthereumRelayerGame;
	type ApproveOrigin = TechnicalCommitteeApproveOrigin;
	type RejectOrigin = EnsureRootOrHalfTechnicalComittee;
	type ConfirmPeriod = ConfirmPeriod;
	type TechnicalMembership = TechnicalMembership;
	type ApproveThreshold = ApproveThreshold;
	type RejectThreshold = RejectThreshold;
	type WeightInfo = ();
}
