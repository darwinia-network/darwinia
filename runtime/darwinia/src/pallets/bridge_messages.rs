pub use pallet_bridge_messages::Instance1 as WithCrabMessages;

// --- darwinia-network ---
use crate::*;
use bp_messages::MessageNonce;
use bp_runtime::{ChainId, CRAB_CHAIN_ID};
use darwinia_fee_market::s2s::{
	FeeMarketMessageAcceptedHandler, FeeMarketMessageConfirmedHandler, FeeMarketPayment,
};
use darwinia_support::evm::{ConcatConverter, IntoAccountId, IntoH160};
use pallet_bridge_messages::Config;

frame_support::parameter_types! {
	pub const MaxMessagesToPruneAtOnce: MessageNonce = 8;
	pub const MaxUnrewardedRelayerEntriesAtInboundLane: MessageNonce =
		bp_crab::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	pub const MaxUnconfirmedMessagesAtInboundLane: MessageNonce =
		bp_crab::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	// `IdentityFee` is used by Darwinia => we may use weight directly
	pub const GetDeliveryConfirmationTransactionFee: Balance =
		bp_crab::MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT as _;
	pub const BridgedChainId: ChainId = CRAB_CHAIN_ID;
	pub RootAccountForPayments: Option<AccountId> = Some(ConcatConverter::<_>::into_account_id((&b"root"[..]).into_h160()));
}

impl Config<WithCrabMessages> for Runtime {
	type Event = Event;
	type WeightInfo = ();
	type Parameter = bm_crab::DarwiniaToCrabMessagesParameter;
	type MaxMessagesToPruneAtOnce = MaxMessagesToPruneAtOnce;
	type MaxUnrewardedRelayerEntriesAtInboundLane = MaxUnrewardedRelayerEntriesAtInboundLane;
	type MaxUnconfirmedMessagesAtInboundLane = MaxUnconfirmedMessagesAtInboundLane;

	type OutboundPayload = bm_crab::ToCrabMessagePayload;
	type OutboundMessageFee = bp_darwinia::Balance;

	type InboundPayload = bm_crab::FromCrabMessagePayload;
	type InboundMessageFee = bp_crab::Balance;
	type InboundRelayer = bp_crab::AccountId;

	type AccountIdConverter = bp_darwinia::AccountIdConverter;

	type TargetHeaderChain = bm_crab::Crab;
	type LaneMessageVerifier = bm_crab::ToCrabMessageVerifier;
	type MessageDeliveryAndDispatchPayment = FeeMarketPayment<
		Runtime,
		WithCrabMessages,
		Ring,
		GetDeliveryConfirmationTransactionFee,
		RootAccountForPayments,
	>;

	type OnMessageAccepted = FeeMarketMessageAcceptedHandler<Self>;
	type OnDeliveryConfirmed = (ToCrabBacking, FeeMarketMessageConfirmedHandler<Self>);

	type SourceHeaderChain = bm_crab::Crab;
	type MessageDispatch = bm_crab::FromCrabMessageDispatch;
	type BridgedChainId = BridgedChainId;
}
