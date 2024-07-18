// This file is part of Darwinia.
//
// Copyright (C) Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod test;

// crates.io
use codec::{Decode, Encode, MaxEncodedLen};
use ethereum::{
	EIP1559Transaction, EIP2930Transaction, LegacyTransaction, TransactionAction,
	TransactionSignature, TransactionV2 as Transaction,
};
use frame_support::sp_runtime::traits::UniqueSaturatedInto;
use scale_info::TypeInfo;
// frontier
use fp_ethereum::{TransactionData, ValidatedTransaction};
use fp_evm::{CheckEvmTransaction, CheckEvmTransactionConfig, TransactionValidationError};
use pallet_evm::{FeeCalculator, GasWeightMapping};
// polkadot-sdk
use frame_support::{traits::EnsureOrigin, PalletError};
use sp_core::{Get, H160, H256, U256};
use sp_runtime::{traits::BadOrigin, DispatchError, RuntimeDebug};
use sp_std::vec::Vec;

pub use pallet::*;

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum ForwardEthOrigin {
	ForwardEth(H160),
}

pub fn ensure_forward_transact<OuterOrigin>(o: OuterOrigin) -> Result<H160, &'static str>
where
	OuterOrigin: Into<Result<ForwardEthOrigin, OuterOrigin>>,
{
	match o.into() {
		Ok(ForwardEthOrigin::ForwardEth(n)) => Ok(n),
		_ => Err("bad origin: expected to be an runtime eth origin"),
	}
}

pub struct EnsureRuntimeEthOrigin;
impl<O: Into<Result<ForwardEthOrigin, O>> + From<ForwardEthOrigin>> EnsureOrigin<O>
	for EnsureRuntimeEthOrigin
{
	type Success = H160;

	fn ensure_origin(o: O) -> Result<Self::Success, BadOrigin> {
		Self::try_origin(o).map_err(|_| BadOrigin)
	}

	fn try_origin(o: O) -> Result<Self::Success, O> {
		o.into().map(|o| match o {
			ForwardEthOrigin::ForwardEth(id) => id,
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<O, ()> {
		Ok(O::from(ForwardEthOrigin::ForwardEth(Default::default())))
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::origin]
	pub type Origin = ForwardEthOrigin;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		/// Handler for applying an already validated transaction
		type ValidatedTransaction: ValidatedTransaction;
		/// Origin for the forward eth transaction
		type ForwardEthOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = H160>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Transaction validation errors.
		ValidationError(TxErrorWrapper),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		OriginFor<T>: Into<Result<ForwardEthOrigin, OriginFor<T>>>,
	{
		//This call can only be used at runtime and is not available to EOA users.
		#[pallet::call_index(0)]
		#[pallet::weight({
			<T as pallet_evm::Config>::GasWeightMapping::gas_to_weight(request.gas_limit.unique_saturated_into(), true)
		})]
		pub fn forward_transact(
			origin: OriginFor<T>,
			request: ForwardRequest,
		) -> DispatchResultWithPostInfo {
			let source = ensure_forward_transact(origin)?;
			let transaction = Self::validated_transaction(source, request)?;

			#[cfg(feature = "evm-tracing")]
			{
				use frame_support::{dispatch::PostDispatchInfo, storage::unhashed};
				use moonbeam_evm_tracer::tracer::EvmTracer;
				use xcm_primitives::{EthereumXcmTracingStatus, ETHEREUM_XCM_TRACING_STORAGE_KEY};

				if let Some(status) = unhashed::get(ETHEREUM_XCM_TRACING_STORAGE_KEY) {
					match status {
						EthereumXcmTracingStatus::Block => {
							EvmTracer::emit_new();
							let mut res = Ok(PostDispatchInfo::default());
							EvmTracer::new().trace(|| {
								res = T::ValidatedTransaction::apply(source, transaction)
									.map(|(post_info, _)| post_info);
							});
							res
						},
						EthereumXcmTracingStatus::Transaction(traced_transaction_hash) =>
							if transaction.hash() == traced_transaction_hash {
								let mut res = Ok(PostDispatchInfo::default());
								EvmTracer::new().trace(|| {
									res = T::ValidatedTransaction::apply(source, transaction)
										.map(|(post_info, _)| post_info);
								});
								unhashed::put::<EthereumXcmTracingStatus>(
									ETHEREUM_XCM_TRACING_STORAGE_KEY,
									&EthereumXcmTracingStatus::TransactionExited,
								);
								res
							} else {
								T::ValidatedTransaction::apply(source, transaction)
									.map(|(post_info, _)| post_info)
							},
						EthereumXcmTracingStatus::TransactionExited => Ok(PostDispatchInfo {
							actual_weight: None,
							pays_fee: frame_support::pallet_prelude::Pays::No,
						}),
					}
				} else {
					T::ValidatedTransaction::apply(source, transaction)
						.map(|(post_info, _)| post_info)
				}
			}
			#[cfg(not(feature = "evm-tracing"))]
			T::ValidatedTransaction::apply(source, transaction).map(|(post_info, _)| post_info)
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Calculates the fee for submitting such an EVM transaction.
	///
	/// The gas_price of an EVM transaction is always the min_gas_price(), which is a fixed value.
	/// Therefore, only the gas_limit and value of the transaction should be considered in the
	/// calculation of the fee, and the gas_price of the transaction itself can be ignored.
	pub fn total_payment(request: &ForwardRequest) -> U256 {
		let base_fee = <T as pallet_evm::Config>::FeeCalculator::min_gas_price().0;
		let fee = base_fee.saturating_mul(request.gas_limit);

		request.value.saturating_add(fee)
	}

	fn validated_transaction(
		source: H160,
		req: ForwardRequest,
	) -> Result<Transaction, DispatchError> {
		let (who, _) = pallet_evm::Pallet::<T>::account_basic(&source);
		let base_fee = T::FeeCalculator::min_gas_price().0;

		let transaction = match req.tx_type {
			TxType::LegacyTransaction => Transaction::Legacy(LegacyTransaction {
				nonce: who.nonce,
				gas_price: base_fee,
				gas_limit: req.gas_limit,
				action: req.action,
				value: req.value,
				input: req.input,
				// copied from:
				// https://github.com/rust-ethereum/ethereum/blob/24739cc8ba6e9d8ee30ada8ec92161e4c48d578e/src/transaction.rs#L798
				signature: TransactionSignature::new(
					38,
					H256([
						190, 103, 224, 160, 125, 182, 125, 168, 212, 70, 247, 106, 221, 89, 14, 84,
						182, 233, 44, 182, 184, 249, 131, 90, 235, 103, 84, 5, 121, 162, 119, 23,
					]),
					H256([
						45, 105, 5, 22, 81, 32, 32, 23, 28, 30, 200, 112, 246, 255, 69, 57, 140,
						200, 96, 146, 80, 50, 107, 232, 153, 21, 251, 83, 142, 123, 215, 24,
					]),
				)
				.expect("This signature is always valid"),
			}),
			TxType::EIP2930Transaction => {
				Transaction::EIP2930(EIP2930Transaction {
					chain_id: 0,
					nonce: who.nonce,
					gas_price: base_fee,
					gas_limit: req.gas_limit,
					action: req.action,
					value: req.value,
					input: req.input,
					access_list: Default::default(),
					// copied from:
					// https://github.com/rust-ethereum/ethereum/blob/24739cc8ba6e9d8ee30ada8ec92161e4c48d578e/src/transaction.rs#L873-L875
					odd_y_parity: false,
					// 36b241b061a36a32ab7fe86c7aa9eb592dd59018cd0443adc0903590c16b02b0
					r: H256([
						54, 178, 65, 176, 97, 163, 106, 50, 171, 127, 232, 108, 122, 169, 235, 89,
						45, 213, 144, 24, 205, 4, 67, 173, 192, 144, 53, 144, 193, 107, 2, 176,
					]),
					// 5edcc541b4741c5cc6dd347c5ed9577ef293a62787b4510465fadbfe39ee4094
					s: H256([
						54, 178, 65, 176, 97, 163, 106, 50, 171, 127, 232, 108, 122, 169, 235, 89,
						45, 213, 144, 24, 205, 4, 67, 173, 192, 144, 53, 144, 193, 107, 2, 176,
					]),
				})
			},
			TxType::EIP1559Transaction => {
				Transaction::EIP1559(EIP1559Transaction {
					chain_id: 0,
					nonce: who.nonce,
					max_fee_per_gas: base_fee,
					max_priority_fee_per_gas: U256::zero(),
					gas_limit: req.gas_limit,
					action: req.action,
					value: req.value,
					input: req.input,
					access_list: Default::default(),
					// copied from:
					// https://github.com/rust-ethereum/ethereum/blob/24739cc8ba6e9d8ee30ada8ec92161e4c48d578e/src/transaction.rs#L873-L875
					odd_y_parity: false,
					// 36b241b061a36a32ab7fe86c7aa9eb592dd59018cd0443adc0903590c16b02b0
					r: H256([
						54, 178, 65, 176, 97, 163, 106, 50, 171, 127, 232, 108, 122, 169, 235, 89,
						45, 213, 144, 24, 205, 4, 67, 173, 192, 144, 53, 144, 193, 107, 2, 176,
					]),
					// 5edcc541b4741c5cc6dd347c5ed9577ef293a62787b4510465fadbfe39ee4094
					s: H256([
						54, 178, 65, 176, 97, 163, 106, 50, 171, 127, 232, 108, 122, 169, 235, 89,
						45, 213, 144, 24, 205, 4, 67, 173, 192, 144, 53, 144, 193, 107, 2, 176,
					]),
				})
			},
		};

		let transaction_data: TransactionData = (&transaction).into();
		let (weight_limit, proof_size_base_cost) =
			match <T as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
				transaction_data.gas_limit.unique_saturated_into(),
				true,
			) {
				weight_limit if weight_limit.proof_size() > 0 =>
					(Some(weight_limit), Some(transaction_data.proof_size_base_cost())),
				_ => (None, None),
			};

		let _ = CheckEvmTransaction::<TxErrorWrapper>::new(
			CheckEvmTransactionConfig {
				evm_config: T::config(),
				block_gas_limit: T::BlockGasLimit::get(),
				base_fee,
				chain_id: T::ChainId::get(),
				is_transactional: true,
			},
			transaction_data.into(),
			weight_limit,
			proof_size_base_cost,
		)
		.validate_in_block_for(&who)
		.and_then(|v| v.with_base_fee())
		.and_then(|v| v.with_balance_for(&who))
		.map_err(|e| <Error<T>>::ValidationError(e))?;

		Ok(transaction)
	}
}

// TODO: replace it with upstream error type
#[derive(Encode, Decode, TypeInfo, PalletError)]
pub enum TxErrorWrapper {
	GasLimitTooLow,
	GasLimitTooHigh,
	GasPriceTooLow,
	PriorityFeeTooHigh,
	BalanceTooLow,
	TxNonceTooLow,
	TxNonceTooHigh,
	InvalidFeeInput,
	InvalidChainId,
	InvalidSignature,
	UnknownError,
}

impl From<TransactionValidationError> for TxErrorWrapper {
	fn from(validation_error: TransactionValidationError) -> Self {
		match validation_error {
			TransactionValidationError::GasLimitTooLow => TxErrorWrapper::GasLimitTooLow,
			TransactionValidationError::GasLimitTooHigh => TxErrorWrapper::GasLimitTooHigh,
			TransactionValidationError::GasPriceTooLow => TxErrorWrapper::GasPriceTooLow,
			TransactionValidationError::PriorityFeeTooHigh => TxErrorWrapper::PriorityFeeTooHigh,
			TransactionValidationError::BalanceTooLow => TxErrorWrapper::BalanceTooLow,
			TransactionValidationError::TxNonceTooLow => TxErrorWrapper::TxNonceTooLow,
			TransactionValidationError::TxNonceTooHigh => TxErrorWrapper::TxNonceTooHigh,
			TransactionValidationError::InvalidFeeInput => TxErrorWrapper::InvalidFeeInput,
			TransactionValidationError::InvalidChainId => TxErrorWrapper::InvalidChainId,
			TransactionValidationError::InvalidSignature => TxErrorWrapper::InvalidSignature,
			TransactionValidationError::UnknownError => TxErrorWrapper::UnknownError,
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct ForwardRequest {
	pub tx_type: TxType,
	pub gas_limit: U256,
	pub action: TransactionAction,
	pub value: U256,
	pub input: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, Default)]
pub enum TxType {
	#[default]
	LegacyTransaction,
	EIP2930Transaction,
	EIP1559Transaction,
}
