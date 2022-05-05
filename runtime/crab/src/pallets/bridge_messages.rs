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
	type Event = Event;
	type WeightInfo = ();
	type Parameter = bm_darwinia::CrabToDarwiniaMessagesParameter;
	type MaxMessagesToPruneAtOnce = MaxMessagesToPruneAtOnce;
	type MaxUnrewardedRelayerEntriesAtInboundLane = MaxUnrewardedRelayerEntriesAtInboundLane;
	type MaxUnconfirmedMessagesAtInboundLane = MaxUnconfirmedMessagesAtInboundLane;

	type OutboundPayload = bm_darwinia::ToDarwiniaMessagePayload;
	type OutboundMessageFee = bp_crab::Balance;

	type InboundPayload = bm_darwinia::FromDarwiniaMessagePayload;
	type InboundMessageFee = bp_darwinia::Balance;
	type InboundRelayer = bp_darwinia::AccountId;

	type AccountIdConverter = bp_crab::AccountIdConverter;

	type TargetHeaderChain = bm_darwinia::Darwinia;
	type LaneMessageVerifier = bm_darwinia::ToDarwiniaMessageVerifier;
	type MessageDeliveryAndDispatchPayment =
		FeeMarketPayment<Self, WithDarwiniaFeeMarket, Ring, RootAccountForPayments>;

	type OnMessageAccepted = FeeMarketMessageAcceptedHandler<Self, WithDarwiniaFeeMarket>;
	type OnDeliveryConfirmed = (
		FromDarwiniaIssuing,
		FeeMarketMessageConfirmedHandler<Self, WithDarwiniaFeeMarket>,
	);

	type SourceHeaderChain = bm_darwinia::Darwinia;
	type MessageDispatch = bm_darwinia::FromDarwiniaMessageDispatch;
	type BridgedChainId = BridgedChainId;
}
