pub use pallet_bridge_messages::{
	Instance1 as WithCrabMessages, Instance2 as WithDarwiniaParachainMessages,
};

// --- darwinia-network ---
use crate::*;
use bp_messages::MessageNonce;
use bp_runtime::{ChainId, CRAB_CHAIN_ID, DARWINIA_PARACHAIN_CHAIN_ID};
use pallet_bridge_messages::Config;
use pallet_fee_market::s2s::{
	FeeMarketMessageAcceptedHandler, FeeMarketMessageConfirmedHandler, FeeMarketPayment,
};

frame_support::parameter_types! {
	// Shared configurations.
	pub const MaxMessagesToPruneAtOnce: MessageNonce = 8;
	// Crab configurations.
	pub const CrabMaxUnrewardedRelayerEntriesAtInboundLane: MessageNonce =
		bp_crab::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	pub const CrabMaxUnconfirmedMessagesAtInboundLane: MessageNonce =
		bp_crab::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	pub const CrabChainId: ChainId = CRAB_CHAIN_ID;
	// Darwinia Parachain configurations.
	pub const DarwiniaParachainMaxUnrewardedRelayerEntriesAtInboundLane: MessageNonce =
		bp_darwinia_parachain::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	pub const DarwiniaParachainMaxUnconfirmedMessagesAtInboundLane: MessageNonce =
		bp_darwinia_parachain::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	pub const DarwiniaParachainChainId: ChainId = DARWINIA_PARACHAIN_CHAIN_ID;
}

impl Config<WithCrabMessages> for Runtime {
	type AccountIdConverter = bp_darwinia::AccountIdConverter;
	type BridgedChainId = CrabChainId;
	type Event = Event;
	type InboundMessageFee = bp_crab::Balance;
	type InboundPayload = bm_crab::FromCrabMessagePayload;
	type InboundRelayer = bp_crab::AccountId;
	type LaneMessageVerifier = bm_crab::ToCrabMessageVerifier;
	type MaxMessagesToPruneAtOnce = MaxMessagesToPruneAtOnce;
	type MaxUnconfirmedMessagesAtInboundLane = CrabMaxUnconfirmedMessagesAtInboundLane;
	type MaxUnrewardedRelayerEntriesAtInboundLane = CrabMaxUnrewardedRelayerEntriesAtInboundLane;
	type MessageDeliveryAndDispatchPayment = FeeMarketPayment<Self, WithCrabFeeMarket, Ring>;
	type MessageDispatch = bm_crab::FromCrabMessageDispatch;
	type OnDeliveryConfirmed = FeeMarketMessageConfirmedHandler<Self, WithCrabFeeMarket>;
	type OnMessageAccepted = FeeMarketMessageAcceptedHandler<Self, WithCrabFeeMarket>;
	type OutboundMessageFee = bp_darwinia::Balance;
	type OutboundPayload = bm_crab::ToCrabMessagePayload;
	type Parameter = bm_crab::DarwiniaToCrabMessagesParameter;
	type SourceHeaderChain = bm_crab::Crab;
	type TargetHeaderChain = bm_crab::Crab;
	type WeightInfo = ();
}
impl Config<WithDarwiniaParachainMessages> for Runtime {
	type AccountIdConverter = bp_darwinia::AccountIdConverter;
	type BridgedChainId = DarwiniaParachainChainId;
	type Event = Event;
	type InboundMessageFee = bp_darwinia_parachain::Balance;
	type InboundPayload = bm_darwinia_parachain::FromDarwiniaParachainMessagePayload;
	type InboundRelayer = bp_darwinia_parachain::AccountId;
	type LaneMessageVerifier = bm_darwinia_parachain::ToDarwiniaParachainMessageVerifier;
	type MaxMessagesToPruneAtOnce = MaxMessagesToPruneAtOnce;
	type MaxUnconfirmedMessagesAtInboundLane = DarwiniaParachainMaxUnconfirmedMessagesAtInboundLane;
	type MaxUnrewardedRelayerEntriesAtInboundLane =
		DarwiniaParachainMaxUnrewardedRelayerEntriesAtInboundLane;
	type MessageDeliveryAndDispatchPayment =
		FeeMarketPayment<Self, WithDarwiniaParachainFeeMarket, Ring>;
	type MessageDispatch = bm_darwinia_parachain::FromDarwiniaParachainMessageDispatch;
	type OnDeliveryConfirmed =
		FeeMarketMessageConfirmedHandler<Self, WithDarwiniaParachainFeeMarket>;
	type OnMessageAccepted = FeeMarketMessageAcceptedHandler<Self, WithDarwiniaParachainFeeMarket>;
	type OutboundMessageFee = bp_darwinia::Balance;
	type OutboundPayload = bm_darwinia_parachain::ToDarwiniaParachainMessagePayload;
	type Parameter = bm_darwinia_parachain::DarwiniaToDarwiniaParachainMessageParameter;
	type SourceHeaderChain = bm_darwinia_parachain::DarwiniaParachain;
	type TargetHeaderChain = bm_darwinia_parachain::DarwiniaParachain;
	type WeightInfo = ();
}
