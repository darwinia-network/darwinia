pub use pallet_bridge_messages::Instance1 as WithCrabMessages;

// --- darwinia-network ---
use crate::*;
use bp_messages::MessageNonce;
use bp_runtime::{ChainId, CRAB_CHAIN_ID};
use darwinia_support::evm::{ConcatConverter, DeriveEthereumAddress, DeriveSubstrateAddress};
use pallet_bridge_messages::Config;
use pallet_fee_market::s2s::{
	FeeMarketMessageAcceptedHandler, FeeMarketMessageConfirmedHandler, FeeMarketPayment,
};

frame_support::parameter_types! {
	pub const MaxMessagesToPruneAtOnce: MessageNonce = 8;
	pub const RootAccountForPayments: Option<AccountId> = Some(ConcatConverter::<_>::derive_substrate_address((&b"root"[..]).derive_ethereum_address()));
	pub const MaxUnrewardedRelayerEntriesAtInboundLane: MessageNonce =
		bp_crab::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	pub const MaxUnconfirmedMessagesAtInboundLane: MessageNonce =
		bp_crab::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	// `IdentityFee` is used by Darwinia => we may use weight directly
	pub const GetDeliveryConfirmationTransactionFee: Balance =
		bp_crab::MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT as _;
	pub const BridgedChainId: ChainId = CRAB_CHAIN_ID;
}

impl Config<WithCrabMessages> for Runtime {
	type AccountIdConverter = bp_darwinia::AccountIdConverter;
	type BridgedChainId = BridgedChainId;
	type Event = Event;
	type InboundMessageFee = bp_crab::Balance;
	type InboundPayload = bm_crab::FromCrabMessagePayload;
	type InboundRelayer = bp_crab::AccountId;
	type LaneMessageVerifier = bm_crab::ToCrabMessageVerifier;
	type MaxMessagesToPruneAtOnce = MaxMessagesToPruneAtOnce;
	type MaxUnconfirmedMessagesAtInboundLane = MaxUnconfirmedMessagesAtInboundLane;
	type MaxUnrewardedRelayerEntriesAtInboundLane = MaxUnrewardedRelayerEntriesAtInboundLane;
	type MessageDeliveryAndDispatchPayment =
		FeeMarketPayment<Self, WithCrabFeeMarket, Ring, RootAccountForPayments>;
	type MessageDispatch = bm_crab::FromCrabMessageDispatch;
	type OnDeliveryConfirmed =
		(ToCrabBacking, FeeMarketMessageConfirmedHandler<Self, WithCrabFeeMarket>);
	type OnMessageAccepted = FeeMarketMessageAcceptedHandler<Self, WithCrabFeeMarket>;
	type OutboundMessageFee = bp_darwinia::Balance;
	type OutboundPayload = bm_crab::ToCrabMessagePayload;
	type Parameter = bm_crab::DarwiniaToCrabMessagesParameter;
	type SourceHeaderChain = bm_crab::Crab;
	type TargetHeaderChain = bm_crab::Crab;
	type WeightInfo = ();
}
