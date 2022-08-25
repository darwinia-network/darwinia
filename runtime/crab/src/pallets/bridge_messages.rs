pub use pallet_bridge_messages::{
	Instance1 as WithDarwiniaMessages, Instance2 as WithCrabParachainMessages,
};

// --- darwinia-network ---
use crate::*;
use bp_messages::MessageNonce;
use bp_runtime::{ChainId, CRAB_PARACHAIN_CHAIN_ID, DARWINIA_CHAIN_ID};
use darwinia_support::evm::{ConcatConverter, DeriveSubstrateAddress};
use pallet_bridge_messages::Config;
use pallet_fee_market::s2s::{
	FeeMarketMessageAcceptedHandler, FeeMarketMessageConfirmedHandler, FeeMarketPayment,
};

frame_support::parameter_types! {
	// Shared configurations.
	pub const MaxMessagesToPruneAtOnce: MessageNonce = 8;
	// 0x726f6f7400000000000000000000000000000000, b"root"
	pub RootAccountForPayments: Option<AccountId> = Some(ConcatConverter::<_>::derive_substrate_address(&H160([0x72, 0x6f, 0x6f, 0x74, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])));
	// Darwinia configurations.
	pub const DarwiniaMaxUnrewardedRelayerEntriesAtInboundLane: MessageNonce =
		bp_darwinia::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	pub const DarwiniaMaxUnconfirmedMessagesAtInboundLane: MessageNonce =
		bp_darwinia::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	pub const DarwiniaChainId: ChainId = DARWINIA_CHAIN_ID;
	// Crab Parachain configurations.
	pub const CrabParachainMaxUnconfirmedMessagesAtInboundLane: MessageNonce =
		bp_crab_parachain::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	pub const CrabParachainMaxUnrewardedRelayerEntriesAtInboundLane: MessageNonce =
		bp_crab_parachain::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	pub const CrabParachainChainId: ChainId = CRAB_PARACHAIN_CHAIN_ID;
}

impl Config<WithDarwiniaMessages> for Runtime {
	type AccountIdConverter = bp_crab::AccountIdConverter;
	type BridgedChainId = DarwiniaChainId;
	type Event = Event;
	type InboundMessageFee = bp_darwinia::Balance;
	type InboundPayload = bm_darwinia::FromDarwiniaMessagePayload;
	type InboundRelayer = bp_darwinia::AccountId;
	type LaneMessageVerifier = bm_darwinia::ToDarwiniaMessageVerifier;
	type MaxMessagesToPruneAtOnce = MaxMessagesToPruneAtOnce;
	type MaxUnconfirmedMessagesAtInboundLane = DarwiniaMaxUnconfirmedMessagesAtInboundLane;
	type MaxUnrewardedRelayerEntriesAtInboundLane =
		DarwiniaMaxUnrewardedRelayerEntriesAtInboundLane;
	type MessageDeliveryAndDispatchPayment =
		FeeMarketPayment<Self, WithDarwiniaFeeMarket, Ring, RootAccountForPayments>;
	type MessageDispatch = bm_darwinia::FromDarwiniaMessageDispatch;
	type OnDeliveryConfirmed =
		(FromDarwiniaIssuing, FeeMarketMessageConfirmedHandler<Self, WithDarwiniaFeeMarket>);
	type OnMessageAccepted = FeeMarketMessageAcceptedHandler<Self, WithDarwiniaFeeMarket>;
	type OutboundMessageFee = bp_crab::Balance;
	type OutboundPayload = bm_darwinia::ToDarwiniaMessagePayload;
	type Parameter = bm_darwinia::CrabToDarwiniaMessagesParameter;
	type SourceHeaderChain = bm_darwinia::Darwinia;
	type TargetHeaderChain = bm_darwinia::Darwinia;
	type WeightInfo = ();
}

impl Config<WithCrabParachainMessages> for Runtime {
	type AccountIdConverter = bp_crab::AccountIdConverter;
	type BridgedChainId = CrabParachainChainId;
	type Event = Event;
	type InboundMessageFee = bp_crab_parachain::Balance;
	type InboundPayload = bm_crab_parachain::FromCrabParachainMessagePayload;
	type InboundRelayer = bp_crab_parachain::AccountId;
	type LaneMessageVerifier = bm_crab_parachain::ToCrabParachainMessageVerifier;
	type MaxMessagesToPruneAtOnce = MaxMessagesToPruneAtOnce;
	type MaxUnconfirmedMessagesAtInboundLane = CrabParachainMaxUnconfirmedMessagesAtInboundLane;
	type MaxUnrewardedRelayerEntriesAtInboundLane =
		CrabParachainMaxUnrewardedRelayerEntriesAtInboundLane;
	type MessageDeliveryAndDispatchPayment =
		FeeMarketPayment<Self, WithCrabParachainFeeMarket, Ring, RootAccountForPayments>;
	type MessageDispatch = bm_crab_parachain::FromCrabParachainMessageDispatch;
	type OnDeliveryConfirmed = (
		ToCrabParachainBacking,
		FeeMarketMessageConfirmedHandler<Self, WithCrabParachainFeeMarket>,
	);
	type OnMessageAccepted = FeeMarketMessageAcceptedHandler<Self, WithCrabParachainFeeMarket>;
	type OutboundMessageFee = bp_crab::Balance;
	type OutboundPayload = bm_crab_parachain::ToCrabParachainMessagePayload;
	type Parameter = bm_crab_parachain::CrabToCrabParachainParameter;
	type SourceHeaderChain = bm_crab_parachain::CrabParachain;
	type TargetHeaderChain = bm_crab_parachain::CrabParachain;
	type WeightInfo = ();
}
