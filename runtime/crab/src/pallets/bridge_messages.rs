pub use pallet_bridge_messages::Instance1 as WithDarwiniaMessages;

// --- paritytech ---
use bp_messages::MessageNonce;
use pallet_bridge_messages::{weights::RialtoWeight, Config};
// --- darwinia-network ---
use crate::{
	darwinia_messages::{
		CrabToDarwiniaMessagesParameter, Darwinia, FromDarwiniaMessageDispatch,
		FromDarwiniaMessagePayload, ToDarwiniaMessagePayload, ToDarwiniaMessageVerifier,
	},
	*,
};
use darwinia_fee_market::s2s::{FeeMarketMessageAcceptedHandler, FeeMarketPayment};
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
	pub const BridgedChainId: bp_runtime::ChainId = DARWINIA_CHAIN_ID;
}

impl Config<WithDarwiniaMessages> for Runtime {
	type Event = Event;
	// FIXME
	type WeightInfo = RialtoWeight<Runtime>;
	type Parameter = CrabToDarwiniaMessagesParameter;
	type MaxMessagesToPruneAtOnce = MaxMessagesToPruneAtOnce;
	type MaxUnrewardedRelayerEntriesAtInboundLane = MaxUnrewardedRelayerEntriesAtInboundLane;
	type MaxUnconfirmedMessagesAtInboundLane = MaxUnconfirmedMessagesAtInboundLane;

	type OutboundPayload = ToDarwiniaMessagePayload;
	type OutboundMessageFee = Balance;

	type InboundPayload = FromDarwiniaMessagePayload;
	type InboundMessageFee = Balance;
	type InboundRelayer = AccountId;

	type AccountIdConverter = AccountIdConverter;

	type TargetHeaderChain = Darwinia;
	type LaneMessageVerifier = ToDarwiniaMessageVerifier<Self>;
	type MessageDeliveryAndDispatchPayment = FeeMarketPayment<
		Runtime,
		WithDarwiniaMessages,
		Ring,
		GetDeliveryConfirmationTransactionFee,
		RootAccountForPayments,
	>;

	type OnMessageAccepted = FeeMarketMessageAcceptedHandler<Self>;
	type OnDeliveryConfirmed = ToDo;
	// type OnDeliveryConfirmed = (
	// 	Substrate2SubstrateIssuing,
	// 	FeeMarketMessageConfirmedHandler<Self>,
	// );

	type SourceHeaderChain = Darwinia;
	type MessageDispatch = FromDarwiniaMessageDispatch;
	type BridgedChainId = BridgedChainId;
}

use bp_messages::{source_chain::OnDeliveryConfirmed, DeliveredMessages, LaneId};

pub struct ToDo;
impl OnDeliveryConfirmed for ToDo {
	fn on_messages_delivered(_lane: &LaneId, _messages: &DeliveredMessages) -> Weight {
		0
	}
}
