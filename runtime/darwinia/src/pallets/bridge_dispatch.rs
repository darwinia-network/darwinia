pub use pallet_bridge_dispatch::{
	Instance1 as WithCrabDispatch, Instance2 as WithDarwiniaParachainDispatch,
};

// --- paritytech ---
use frame_support::{
	ensure,
	traits::{OriginTrait, WithdrawReasons},
};
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidityError};
// --- darwinia-network ---
use crate::*;
use bp_message_dispatch::{CallValidate, IntoDispatchOrigin as IntoDispatchOriginT};
use bp_messages::{LaneId, MessageNonce};
use darwinia_ethereum::{RawOrigin, Transaction};
use darwinia_evm::CurrencyAdapt;
use darwinia_support::evm::{decimal_convert, DeriveEthereumAddress, DeriveSubstrateAddress};
use pallet_bridge_dispatch::Config;

frame_support::parameter_types! {
	pub const MaxUsableBalanceFromRelayer: Balance = 100 * COIN;
}

pub struct CallValidator;
impl CallValidate<bp_darwinia::AccountId, Origin, Call> for CallValidator {
	fn check_receiving_before_dispatch(
		relayer_account: &bp_darwinia::AccountId,
		call: &Call,
	) -> Result<(), &'static str> {
		match call {
			Call::Ethereum(darwinia_ethereum::Call::message_transact {
				transaction: Transaction::Legacy(t),
			}) => {
				// Use fixed gas price now.
				let gas_price = <Runtime as darwinia_evm::Config>::FeeCalculator::min_gas_price();
				let fee = t.gas_limit.saturating_mul(gas_price);

				// Ensure the relayer's account has enough balance to withdraw. If not,
				// reject the call before dispatch.
				Ok(<Runtime as darwinia_evm::Config>::RingBalanceAdapter::ensure_can_withdraw(
					relayer_account,
					fee.min(decimal_convert(MaxUsableBalanceFromRelayer::get(), None)),
					WithdrawReasons::all(),
				)
				.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Payment))?)
			},
			_ => Ok(()),
		}
	}

	fn call_validate(
		relayer_account: &bp_darwinia::AccountId,
		origin: &Origin,
		call: &Call,
	) -> Result<(), TransactionValidityError> {
		match call {
			// Note: Only support Ethereum::message_transact(LegacyTransaction)
			Call::Ethereum(darwinia_ethereum::Call::message_transact { transaction: tx }) => {
				match origin.caller() {
					OriginCaller::Ethereum(RawOrigin::EthereumTransaction(id)) => match tx {
						Transaction::Legacy(t) => {
							// Only non-payable call supported.
							ensure!(
								t.value.is_zero(),
								TransactionValidityError::Invalid(InvalidTransaction::Payment,)
							);

							// Use fixed gas price now.
							let gas_price =
								<Runtime as darwinia_evm::Config>::FeeCalculator::min_gas_price();
							let fee = t.gas_limit.saturating_mul(gas_price);

							// MaxUsableBalanceFromRelayer is the cap limitation for fee in case
							// gas_limit is too large for relayer
							ensure!(
								fee <= decimal_convert(MaxUsableBalanceFromRelayer::get(), None),
								TransactionValidityError::Invalid(InvalidTransaction::Custom(2))
							);

							// Already done `evm_ensure_can_withdraw` in
							// check_receiving_before_dispatch
							let derived_substrate_address =
								<Runtime as darwinia_evm::Config>::IntoAccountId::derive_substrate_address(id);

							<Runtime as darwinia_evm::Config>::RingBalanceAdapter::evm_transfer(
								relayer_account,
								&derived_substrate_address,
								fee,
							)
							.map_err(|_| {
								TransactionValidityError::Invalid(InvalidTransaction::Custom(3))
							})
						},
						_ => Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(1))),
					},
					_ => Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(0))),
				}
			},
			Call::System(frame_system::Call::remark { .. })
			| Call::System(frame_system::Call::remark_with_event { .. }) => Ok(()),
			_ => Err(TransactionValidityError::Invalid(InvalidTransaction::Call)),
		}
	}
}

pub struct IntoDispatchOrigin;
impl IntoDispatchOriginT<bp_darwinia::AccountId, Call, Origin> for IntoDispatchOrigin {
	fn into_dispatch_origin(id: &bp_darwinia::AccountId, call: &Call) -> Origin {
		match call {
			Call::Ethereum(darwinia_ethereum::Call::message_transact { .. }) => {
				let derive_eth_address = id.derive_ethereum_address();
				darwinia_ethereum::RawOrigin::EthereumTransaction(derive_eth_address).into()
			},
			_ => frame_system::RawOrigin::Signed(id.clone()).into(),
		}
	}
}

impl Config<WithCrabDispatch> for Runtime {
	type AccountIdConverter = bp_darwinia::AccountIdConverter;
	type BridgeMessageId = (LaneId, MessageNonce);
	type Call = Call;
	type CallValidator = CallValidator;
	type EncodedCall = bm_crab::FromCrabEncodedCall;
	type Event = Event;
	type IntoDispatchOrigin = IntoDispatchOrigin;
	type SourceChainAccountId = bp_crab::AccountId;
	type TargetChainAccountPublic = bp_darwinia::AccountPublic;
	type TargetChainSignature = bp_darwinia::Signature;
}
impl Config<WithDarwiniaParachainDispatch> for Runtime {
	type AccountIdConverter = bp_darwinia::AccountIdConverter;
	type BridgeMessageId = (LaneId, MessageNonce);
	type Call = Call;
	type CallValidator = CallValidator;
	type EncodedCall = bm_darwinia_parachain::FromDarwiniaParachainEncodedCall;
	type Event = Event;
	type IntoDispatchOrigin = IntoDispatchOrigin;
	type SourceChainAccountId = bp_darwinia_parachain::AccountId;
	type TargetChainAccountPublic = bp_darwinia::AccountPublic;
	type TargetChainSignature = bp_darwinia_parachain::Signature;
}
