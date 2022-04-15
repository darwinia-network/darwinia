// --- paritytech ---
use frame_support::PalletId;
use sp_runtime::AccountId32;
// --- darwinia-network ---
use crate::*;
use bp_messages::LaneId;
use bp_runtime::{ChainId, DARWINIA_CHAIN_ID};
use bridge_runtime_common::lanes::DARWINIA_CRAB_LANE;
use darwinia_support::{s2s::ToEthAddress, ChainName};
use from_substrate_issuing::Config;

// Convert from AccountId32 to H160
pub struct TruncateToEthAddress;
impl ToEthAddress<AccountId32> for TruncateToEthAddress {
	fn into_ethereum_id(address: &AccountId32) -> H160 {
		let account20: &[u8] = &address.as_ref();

		H160::from_slice(&account20[..20])
	}
}

frame_support::parameter_types! {
	pub const S2sIssuingPalletId: PalletId = PalletId(*b"da/fdais");
	pub const DarwiniaChainId: ChainId = DARWINIA_CHAIN_ID;
	pub const BridgeDarwiniaLaneId: LaneId = DARWINIA_CRAB_LANE;
	pub BackingChainName: ChainName = (b"Darwinia").to_vec();
}

impl Config for Runtime {
	type PalletId = S2sIssuingPalletId;
	type Event = Event;
	type WeightInfo = ();
	type RingCurrency = Ring;
	type BridgedAccountIdConverter = bp_crab::AccountIdConverter;
	type BridgedChainId = DarwiniaChainId;
	type ToEthAddressT = TruncateToEthAddress;
	type OutboundPayloadCreator = bm_darwinia::ToDarwiniaOutboundPayLoad;
	type InternalTransactHandler = Ethereum;
	type BackingChainName = BackingChainName;
	type MessageLaneId = BridgeDarwiniaLaneId;
}
