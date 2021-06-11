#[cfg(feature = "std")]
pub use darwinia_ethereum_relay::DagsMerkleRootsLoader;

// --- substrate ---
use frame_support::PalletId;
use sp_runtime::Perbill;
// --- darwinia ---
use crate::*;
use darwinia_ethereum_relay::Config;
use ethereum_primitives::EthereumNetworkType;

frame_support::parameter_types! {
	pub const EthereumRelayPalletId: PalletId = PalletId(*b"da/ethrl");
	pub const EthereumNetwork: EthereumNetworkType = EthereumNetworkType::Mainnet;
	pub const ConfirmPeriod: BlockNumber = 3 * DAYS;
	pub const ApproveThreshold: Perbill = Perbill::from_percent(60);
	pub const RejectThreshold: Perbill = Perbill::from_percent(1);
}

impl Config for Runtime {
	type PalletId = EthereumRelayPalletId;
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
