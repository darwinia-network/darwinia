pub use pallet_bridge_messages::Instance1 as WithCrabMessages;

// --- paritytech ---
use bp_messages::MessageNonce;
use bp_runtime::ChainId;
use pallet_bridge_messages::{weights::RialtoWeight, Config};
// --- darwinia-network ---
use crate::{
	messages::crab_messages::{
		Crab, DarwiniaToCrabMessagesParameter, FromCrabMessageDispatch, FromCrabMessagePayload,
		ToCrabMessagePayload, ToCrabMessageVerifier,
	},
	*,
};
use darwinia_fee_market::s2s::{
	FeeMarketMessageAcceptedHandler, FeeMarketMessageConfirmedHandler, FeeMarketPayment,
};
use darwinia_support::evm::{ConcatConverter, IntoAccountId, IntoH160};

frame_support::parameter_types! {
	pub const MaxMessagesToPruneAtOnce: MessageNonce = 8;
	pub const MaxUnrewardedRelayerEntriesAtInboundLane: MessageNonce =
		MAX_UNREWARDED_RELAYER_ENTRIES_AT_INBOUND_LANE;
	pub const MaxUnconfirmedMessagesAtInboundLane: MessageNonce =
		MAX_UNCONFIRMED_MESSAGES_AT_INBOUND_LANE;
	// `IdentityFee` is used by Darwinia => we may use weight directly
	pub const GetDeliveryConfirmationTransactionFee: Balance =
		MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT as _;
	pub RootAccountForPayments: Option<AccountId> = Some(ConcatConverter::<_>::into_account_id((&b"root"[..]).into_h160()));
	pub const BridgedChainId: ChainId = CRAB_CHAIN_ID;
}

impl Config<WithCrabMessages> for Runtime {
	type Event = Event;
	// FIXME
	type WeightInfo = RialtoWeight<Runtime>;
	type Parameter = DarwiniaToCrabMessagesParameter;
	type MaxMessagesToPruneAtOnce = MaxMessagesToPruneAtOnce;
	type MaxUnrewardedRelayerEntriesAtInboundLane = MaxUnrewardedRelayerEntriesAtInboundLane;
	type MaxUnconfirmedMessagesAtInboundLane = MaxUnconfirmedMessagesAtInboundLane;

	type OutboundPayload = ToCrabMessagePayload;
	type OutboundMessageFee = Balance;

	type InboundPayload = FromCrabMessagePayload;
	type InboundMessageFee = Balance;
	type InboundRelayer = AccountId;

	type AccountIdConverter = AccountIdConverter;

	type TargetHeaderChain = Crab;
	type LaneMessageVerifier = ToCrabMessageVerifier<Self>;
	type MessageDeliveryAndDispatchPayment = FeeMarketPayment<
		Runtime,
		WithCrabMessages,
		Ring,
		GetDeliveryConfirmationTransactionFee,
		RootAccountForPayments,
	>;

	type OnMessageAccepted = FeeMarketMessageAcceptedHandler<Self>;
	type OnDeliveryConfirmed = FeeMarketMessageConfirmedHandler<Self>;

	type SourceHeaderChain = Crab;
	type MessageDispatch = FromCrabMessageDispatch;
	type BridgedChainId = BridgedChainId;
}
