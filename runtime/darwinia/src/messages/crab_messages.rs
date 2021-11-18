//! Everything required to serve Darwinia <-> Crab messages.

// // --- crates.io ---
// use codec::{Decode, Encode};
// // --- paritytech ---
// use bp_message_dispatch::CallOrigin;
// use bp_messages::{
// 	source_chain::TargetHeaderChain,
// 	target_chain::{ProvedMessages, SourceHeaderChain},
// 	InboundLaneData, LaneId, Message, MessageNonce, Parameter as MessagesParameter,
// };
// use bp_runtime::{messages::DispatchFeePayment, ChainId};
use bridge_runtime_common::messages::{
	self,
	source::{self, FromBridgedChainMessagesDeliveryProof, FromThisChainMessagePayload},
	target::{
		self, FromBridgedChainEncodedMessageCall, FromBridgedChainMessageDispatch,
		FromBridgedChainMessagePayload, FromBridgedChainMessagesProof,
	},
	MessageBridge, MessageTransaction,
};
// use frame_support::{
// 	weights::{DispatchClass, Weight},
// 	RuntimeDebug,
// };
// use pallet_bridge_messages::EXPECTED_DEFAULT_MESSAGE_LENGTH;
// use sp_runtime::{traits::Zero, FixedPointNumber, FixedU128, MultiSignature, MultiSigner};
// use sp_std::{convert::TryFrom, ops::RangeInclusive};
// --- darwinia-network ---
use crate::Call;
// use bridge_primitives::{
// 	DarwiniaFromThisChainMessageVerifier, CRAB_CHAIN_ID, DARWINIA_CHAIN_ID,
// 	DARWINIA_CRAB_LANE, WITH_DARWINIA_MESSAGES_PALLET_NAME,
// };
// use dp_s2s::{CallParams, CreatePayload};

// /// Message payload for Darwinia -> Crab messages.
// pub type ToCrabMessagePayload = FromThisChainMessagePayload<WithCrabMessageBridge>;
// /// The s2s issuing pallet index in the pangolin chain runtime
// pub const CRAB_S2S_ISSUING_PALLET_INDEX: u8 = 49;

// #[derive(RuntimeDebug, Encode, Decode, Clone, PartialEq, Eq)]
// pub struct ToCrabOutboundPayload;

// impl CreatePayload<AccountId, MultiSigner, MultiSignature> for ToCrabOutboundPayload {
// 	type Payload = ToCrabMessagePayload;

// 	fn create(
// 		origin: CallOrigin<AccountId, MultiSigner, MultiSignature>,
// 		spec_version: u32,
// 		weight: u64,
// 		call_params: CallParams,
// 		dispatch_fee_payment: DispatchFeePayment,
// 	) -> Result<Self::Payload, &'static str> {
// 		let call = Self::encode_call(CRAB_S2S_ISSUING_PALLET_INDEX, call_params)?;
// 		return Ok(ToCrabMessagePayload {
// 			spec_version,
// 			weight,
// 			origin,
// 			call,
// 			dispatch_fee_payment,
// 		});
// 	}
// }

// /// Message verifier for Darwinia -> Crab messages.
// pub type ToCrabMessageVerifier<R> =
// 	DarwiniaFromThisChainMessageVerifier<WithCrabMessageBridge, R>;
// /// Message payload for Crab -> Darwinia messages.
// pub type FromCrabMessagePayload = FromBridgedChainMessagePayload<WithCrabMessageBridge>;
/// Encoded Darwinia Call as it comes from Crab.
pub type FromCrabEncodedCall = FromBridgedChainEncodedMessageCall<Call>;
// /// Messages proof for Crab -> Darwinia messages.
// type FromCrabMessagesProof = FromBridgedChainMessagesProof<pangolin_primitives::Hash>;
// /// Messages delivery proof for Darwinia -> Crab messages.
// type ToCrabMessagesDeliveryProof =
// 	FromBridgedChainMessagesDeliveryProof<pangolin_primitives::Hash>;
// /// Call-dispatch based message dispatch for Crab -> Darwinia messages.
// pub type FromCrabMessageDispatch =
// 	FromBridgedChainMessageDispatch<WithCrabMessageBridge, Runtime, Ring, WithCrabDispatch>;

// /// Initial value of `CrabToDarwiniaConversionRate` parameter.
// pub const INITIAL_CRAB_TO_DARWINIA_CONVERSION_RATE: FixedU128 =
// 	FixedU128::from_inner(FixedU128::DIV);

// frame_support::parameter_types! {
// 	/// Crab to Darwinia conversion rate. Initially we treat both tokens as equal.
// 	pub storage CrabToDarwiniaConversionRate: FixedU128 = INITIAL_CRAB_TO_DARWINIA_CONVERSION_RATE;
// }

// /// Darwinia -> Crab message lane pallet parameters.
// #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
// pub enum DarwiniaToCrabMessagesParameter {
// 	/// The conversion formula we use is: `DarwiniaTokens = CrabTokens * conversion_rate`.
// 	CrabToDarwiniaConversionRate(FixedU128),
// }
// impl MessagesParameter for DarwiniaToCrabMessagesParameter {
// 	fn save(&self) {
// 		match *self {
// 			DarwiniaToCrabMessagesParameter::CrabToDarwiniaConversionRate(
// 				ref conversion_rate,
// 			) => CrabToDarwiniaConversionRate::set(conversion_rate),
// 		}
// 	}
// }

// /// Darwinia <-> Crab message bridge.
// #[derive(Clone, Copy, RuntimeDebug)]
// pub struct WithCrabMessageBridge;
// impl MessageBridge for WithCrabMessageBridge {
// 	const RELAYER_FEE_PERCENT: u32 = 10;
// 	const THIS_CHAIN_ID: ChainId = DARWINIA_CHAIN_ID;
// 	const BRIDGED_CHAIN_ID: ChainId = CRAB_CHAIN_ID;
// 	const BRIDGED_MESSAGES_PALLET_NAME: &'static str = WITH_DARWINIA_MESSAGES_PALLET_NAME;

// 	type ThisChain = Darwinia;
// 	type BridgedChain = Crab;

// 	fn bridged_balance_to_this_balance(
// 		bridged_balance: pangolin_primitives::Balance,
// 	) -> pangoro_primitives::Balance {
// 		pangoro_primitives::Balance::try_from(
// 			CrabToDarwiniaConversionRate::get().saturating_mul_int(bridged_balance),
// 		)
// 		.unwrap_or(pangoro_primitives::Balance::MAX)
// 	}
// }

// /// Darwinia chain from message lane point of view.
// #[derive(Clone, Copy, RuntimeDebug)]
// pub struct Darwinia;
// impl messages::ChainWithMessages for Darwinia {
// 	type Hash = pangoro_primitives::Hash;
// 	type AccountId = pangoro_primitives::AccountId;
// 	type Signer = pangoro_primitives::AccountPublic;
// 	type Signature = pangoro_primitives::Signature;
// 	type Weight = Weight;
// 	type Balance = pangoro_primitives::Balance;
// }
// impl messages::ThisChainWithMessages for Darwinia {
// 	type Call = Call;

// 	fn is_outbound_lane_enabled(lane: &LaneId) -> bool {
// 		*lane == [0, 0, 0, 0] || *lane == [0, 0, 0, 1] || *lane == DARWINIA_CRAB_LANE
// 	}

// 	fn maximal_pending_messages_at_outbound_lane() -> MessageNonce {
// 		MessageNonce::MAX
// 	}

// 	fn estimate_delivery_confirmation_transaction() -> MessageTransaction<Weight> {
// 		let inbound_data_size =
// 			InboundLaneData::<pangoro_primitives::AccountId>::encoded_size_hint(
// 				bridge_primitives::MAXIMAL_ENCODED_ACCOUNT_ID_SIZE,
// 				1,
// 				1,
// 			)
// 			.unwrap_or(u32::MAX);

// 		MessageTransaction {
// 			dispatch_weight: bridge_primitives::MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT,
// 			size: inbound_data_size
// 				.saturating_add(bridge_primitives::EXTRA_STORAGE_PROOF_SIZE)
// 				.saturating_add(bridge_primitives::TX_EXTRA_BYTES),
// 		}
// 	}

// 	fn transaction_payment(transaction: MessageTransaction<Weight>) -> pangoro_primitives::Balance {
// 		// in our testnets, both per-byte fee and weight-to-fee are 1:1
// 		messages::transaction_payment(
// 			pangoro_runtime_system_params::RuntimeBlockWeights::get()
// 				.get(DispatchClass::Normal)
// 				.base_extrinsic,
// 			1,
// 			FixedU128::zero(),
// 			|weight| weight as _,
// 			transaction,
// 		)
// 	}
// }

// /// Crab chain from message lane point of view.
// #[derive(Clone, Copy, RuntimeDebug)]
// pub struct Crab;
// impl messages::ChainWithMessages for Crab {
// 	type Hash = pangolin_primitives::Hash;
// 	type AccountId = pangolin_primitives::AccountId;
// 	type Signer = pangolin_primitives::AccountPublic;
// 	type Signature = pangolin_primitives::Signature;
// 	type Weight = Weight;
// 	type Balance = pangolin_primitives::Balance;
// }
// impl messages::BridgedChainWithMessages for Crab {
// 	fn maximal_extrinsic_size() -> u32 {
// 		pangolin_runtime_system_params::max_extrinsic_size()
// 	}

// 	fn message_weight_limits(_message_payload: &[u8]) -> RangeInclusive<Weight> {
// 		// we don't want to relay too large messages + keep reserve for future upgrades
// 		let upper_limit = messages::target::maximal_incoming_message_dispatch_weight(
// 			pangolin_runtime_system_params::max_extrinsic_weight(),
// 		);

// 		// we're charging for payload bytes in `WithCrabMessageBridge::transaction_payment` function
// 		//
// 		// this bridge may be used to deliver all kind of messages, so we're not making any assumptions about
// 		// minimal dispatch weight here

// 		0..=upper_limit
// 	}

// 	fn estimate_delivery_transaction(
// 		message_payload: &[u8],
// 		include_pay_dispatch_fee_cost: bool,
// 		message_dispatch_weight: Weight,
// 	) -> MessageTransaction<Weight> {
// 		let message_payload_len = u32::try_from(message_payload.len()).unwrap_or(u32::MAX);
// 		let extra_bytes_in_payload = Weight::from(message_payload_len)
// 			.saturating_sub(EXPECTED_DEFAULT_MESSAGE_LENGTH.into());

// 		MessageTransaction {
// 			dispatch_weight: extra_bytes_in_payload
// 				.saturating_mul(bridge_primitives::ADDITIONAL_MESSAGE_BYTE_DELIVERY_WEIGHT)
// 				.saturating_add(bridge_primitives::DEFAULT_MESSAGE_DELIVERY_TX_WEIGHT)
// 				.saturating_add(message_dispatch_weight)
// 				.saturating_sub(if include_pay_dispatch_fee_cost {
// 					0
// 				} else {
// 					bridge_primitives::PAY_INBOUND_DISPATCH_FEE_WEIGHT
// 				}),
// 			size: message_payload_len
// 				.saturating_add(bridge_primitives::EXTRA_STORAGE_PROOF_SIZE)
// 				.saturating_add(bridge_primitives::TX_EXTRA_BYTES),
// 		}
// 	}

// 	fn transaction_payment(
// 		transaction: MessageTransaction<Weight>,
// 	) -> pangolin_primitives::Balance {
// 		// in our testnets, both per-byte fee and weight-to-fee are 1:1
// 		messages::transaction_payment(
// 			pangolin_runtime_system_params::RuntimeBlockWeights::get()
// 				.get(DispatchClass::Normal)
// 				.base_extrinsic,
// 			1,
// 			FixedU128::zero(),
// 			|weight| weight as _,
// 			transaction,
// 		)
// 	}
// }
// impl TargetHeaderChain<ToCrabMessagePayload, pangolin_primitives::AccountId> for Crab {
// 	type Error = &'static str;
// 	// The proof is:
// 	// - hash of the header this proof has been created with;
// 	// - the storage proof or one or several keys;
// 	// - id of the lane we prove state of.
// 	type MessagesDeliveryProof = ToCrabMessagesDeliveryProof;

// 	fn verify_message(payload: &ToCrabMessagePayload) -> Result<(), Self::Error> {
// 		source::verify_chain_message::<WithCrabMessageBridge>(payload)
// 	}

// 	fn verify_messages_delivery_proof(
// 		proof: Self::MessagesDeliveryProof,
// 	) -> Result<(LaneId, InboundLaneData<pangoro_primitives::AccountId>), Self::Error> {
// 		source::verify_messages_delivery_proof::<
// 			WithCrabMessageBridge,
// 			Runtime,
// 			WithCrabGrandpa,
// 		>(proof)
// 	}
// }
// impl SourceHeaderChain<pangolin_primitives::Balance> for Crab {
// 	type Error = &'static str;
// 	// The proof is:
// 	// - hash of the header this proof has been created with;
// 	// - the storage proof or one or several keys;
// 	// - id of the lane we prove messages for;
// 	// - inclusive range of messages nonces that are proved.
// 	type MessagesProof = FromCrabMessagesProof;

// 	fn verify_messages_proof(
// 		proof: Self::MessagesProof,
// 		messages_count: u32,
// 	) -> Result<ProvedMessages<Message<pangolin_primitives::Balance>>, Self::Error> {
// 		target::verify_messages_proof::<WithCrabMessageBridge, Runtime, WithCrabGrandpa>(
// 			proof,
// 			messages_count,
// 		)
// 	}
// }
