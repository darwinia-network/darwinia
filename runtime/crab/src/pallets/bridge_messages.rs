pub use pallet_bridge_messages::Instance1 as WithDarwiniaMessages;

// --- darwinia-network ---
use crate::*;
use bp_messages::MessageNonce;
use bp_runtime::{ChainId, DARWINIA_CHAIN_ID};
use darwinia_fee_market::s2s::{
	FeeMarketMessageAcceptedHandler, FeeMarketMessageConfirmedHandler, FeeMarketPayment,
};
use darwinia_support::evm::{ConcatConverter, IntoAccountId, IntoH160};
use pallet_bridge_messages::Config;

frame_support::parameter_types! {
	pub const MaxMessagesToPruneAtOnce: MessageNonce = 8;
	pub const MaxUnrewardedRelayerEntriesAtInboundLane: MessageNonce =
		bp_darwinia::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	pub const MaxUnconfirmedMessagesAtInboundLane: MessageNonce =
		bp_darwinia::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	// `IdentityFee` is used by Darwinia => we may use weight directly
	pub const GetDeliveryConfirmationTransactionFee: Balance =
		bp_darwinia::MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT as _;
	pub const BridgedChainId: ChainId = DARWINIA_CHAIN_ID;
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
	type MaxUnconfirmedMessagesAtInboundLane = MaxUnconfirmedMessagesAtInboundLane;
	type MaxUnrewardedRelayerEntriesAtInboundLane = MaxUnrewardedRelayerEntriesAtInboundLane;
	type MessageDeliveryAndDispatchPayment = FeeMarketPayment<
		Runtime,
		WithDarwiniaMessages,
		Ring,
		GetDeliveryConfirmationTransactionFee,
		RootAccountForPayments,
	>;
	type MessageDispatch = bm_darwinia::FromDarwiniaMessageDispatch;
	type OnDeliveryConfirmed = (FromDarwiniaIssuing, FeeMarketMessageConfirmedHandler<Self>);
	type OnMessageAccepted = FeeMarketMessageAcceptedHandler<Self>;
	type OutboundMessageFee = bp_crab::Balance;
	type OutboundPayload = bm_darwinia::ToDarwiniaMessagePayload;
	type Parameter = bm_darwinia::CrabToDarwiniaMessagesParameter;
	type SourceHeaderChain = bm_darwinia::Darwinia;
	type TargetHeaderChain = bm_darwinia::Darwinia;
	type WeightInfo = ();
}
