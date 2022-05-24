pub use pallet_bridge_messages::{
	Instance1 as WithDarwiniaMessages, Instance2 as WithCrabParachainMessages,
};

// --- darwinia-network ---
use crate::*;
use bp_messages::MessageNonce;
use bp_runtime::{ChainId, CRAB_PARACHAIN_CHAIN_ID, DARWINIA_CHAIN_ID};
use darwinia_support::evm::{ConcatConverter, IntoAccountId, IntoH160};
use pallet_bridge_messages::Config;
use pallet_fee_market::s2s::{
	FeeMarketMessageAcceptedHandler, FeeMarketMessageConfirmedHandler, FeeMarketPayment,
};

frame_support::parameter_types! {
	pub const MaxMessagesToPruneAtOnce: MessageNonce = 8;
	pub const DarwiniaMaxUnrewardedRelayerEntriesAtInboundLane: MessageNonce =
		bp_darwinia::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	pub const DarwiniaMaxUnconfirmedMessagesAtInboundLane: MessageNonce =
		bp_darwinia::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;

	pub const CrabParachainMaxUnconfirmedMessagesAtInboundLane: MessageNonce =
		bp_crab_parachain::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	pub const CrabParachainMaxUnrewardedRelayerEntriesAtInboundLane: MessageNonce =
		bp_crab_parachain::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;

	// `IdentityFee` is used by Darwinia => we may use weight directly
	pub const GetDeliveryConfirmationTransactionFee: Balance =
		bp_darwinia::MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT as _;
	pub const BridgedChainId: ChainId = DARWINIA_CHAIN_ID;
	pub const CrabParachainChainId: ChainId = CRAB_PARACHAIN_CHAIN_ID;
	pub RootAccountForPayments: Option<AccountId> = Some(ConcatConverter::<_>::into_account_id((&b"root"[..]).into_h160()));
}

impl Config<WithDarwiniaMessages> for Runtime {
	type AccountIdConverter = bp_crab::AccountIdConverter;
	type BridgedChainId = BridgedChainId;
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
	type OnDeliveryConfirmed =
		(FromDarwiniaIssuing, FeeMarketMessageConfirmedHandler<Self, WithCrabParachainFeeMarket>);
	type OnMessageAccepted = FeeMarketMessageAcceptedHandler<Self, WithCrabParachainFeeMarket>;
	type OutboundMessageFee = Balance;
	type OutboundPayload = bm_crab_parachain::ToCrabParachainMessagePayload;
	type Parameter = bm_crab_parachain::CrabToCrabParachainParameter;
	type SourceHeaderChain = bm_crab_parachain::CrabParachain;
	type TargetHeaderChain = bm_crab_parachain::CrabParachain;
	type WeightInfo = ();
}
