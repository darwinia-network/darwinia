// This file is part of Darwinia.
//
// Copyright (C) 2018-2023 Darwinia Network
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
#![deny(unused_crate_dependencies)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

// core
use core::borrow::BorrowMut;
// crates.io
use codec::{Decode, Encode, MaxEncodedLen};
use ethereum::TransactionV2 as Transaction;
use frame_support::sp_runtime::traits::UniqueSaturatedInto;
use scale_info::TypeInfo;
// frontier
use fp_ethereum::{TransactionData, ValidatedTransaction};
use fp_evm::{CheckEvmTransaction, CheckEvmTransactionConfig, TransactionValidationError};
use pallet_evm::{FeeCalculator, GasWeightMapping};
// substrate
use frame_support::{traits::EnsureOrigin, PalletError};
use sp_core::{H160, U256};
use sp_runtime::{traits::BadOrigin, RuntimeDebug};
use sp_std::boxed::Box;

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
		/// EVM validation errors.
		MessageTransactError(EvmTxErrorWrapper),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		OriginFor<T>: Into<Result<ForwardEthOrigin, OriginFor<T>>>,
	{
		//This call can only be used at runtime and is not available to EOA users.
		#[pallet::call_index(0)]
		#[pallet::weight({
			<T as pallet_evm::Config>::GasWeightMapping::gas_to_weight({
				let transaction_data: TransactionData = (&**transaction).into();
				transaction_data.gas_limit.unique_saturated_into()
			}, true)
		})]
		pub fn forward_transact(
			origin: OriginFor<T>,
			mut transaction: Box<Transaction>,
		) -> DispatchResultWithPostInfo {
			let source = ensure_forward_transact(origin)?;
			let (who, _) = pallet_evm::Pallet::<T>::account_basic(&source);
			let base_fee = T::FeeCalculator::min_gas_price().0;

			let transaction_mut = transaction.borrow_mut();
			match transaction_mut {
				Transaction::Legacy(tx) => {
					tx.nonce = who.nonce;
					tx.gas_price = base_fee;
				},
				Transaction::EIP2930(tx) => {
					tx.nonce = who.nonce;
					tx.gas_price = base_fee;
				},
				Transaction::EIP1559(tx) => {
					tx.nonce = who.nonce;
					tx.max_fee_per_gas = base_fee;
					tx.max_priority_fee_per_gas = U256::zero();
				},
			};

			let transaction_data: TransactionData = (&*transaction).into();
			let (weight_limit, proof_size_base_cost) =
				match <T as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
					transaction_data.gas_limit.unique_saturated_into(),
					true,
				) {
					weight_limit if weight_limit.proof_size() > 0 =>
						(Some(weight_limit), Some(proof_size_base_cost(&transaction))),
					_ => (None, None),
				};

			let _ = CheckEvmTransaction::<EvmTxErrorWrapper>::new(
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
			.map_err(|e| <Error<T>>::MessageTransactError(e))?;

			T::ValidatedTransaction::apply(source, *transaction).map(|(post_info, _)| post_info)
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Calculates the fee for submitting such an EVM transaction.
	///
	/// The gas_price of an EVM transaction is always the min_gas_price(), which is a fixed value.
	/// Therefore, only the gas_limit and value of the transaction should be considered in the
	/// calculation of the fee, and the gas_price of the transaction itself can be ignored.
	pub fn total_payment(tx_data: TransactionData) -> U256 {
		let base_fee = <T as pallet_evm::Config>::FeeCalculator::min_gas_price().0;
		let fee = base_fee.saturating_mul(tx_data.gas_limit);

		tx_data.value.saturating_add(fee)
	}
}

// TODO: replace it with upstream error type
#[derive(Encode, Decode, TypeInfo, PalletError)]
pub enum EvmTxErrorWrapper {
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

impl From<TransactionValidationError> for EvmTxErrorWrapper {
	fn from(validation_error: TransactionValidationError) -> Self {
		match validation_error {
			TransactionValidationError::GasLimitTooLow => EvmTxErrorWrapper::GasLimitTooLow,
			TransactionValidationError::GasLimitTooHigh => EvmTxErrorWrapper::GasLimitTooHigh,
			TransactionValidationError::GasPriceTooLow => EvmTxErrorWrapper::GasPriceTooLow,
			TransactionValidationError::PriorityFeeTooHigh => EvmTxErrorWrapper::PriorityFeeTooHigh,
			TransactionValidationError::BalanceTooLow => EvmTxErrorWrapper::BalanceTooLow,
			TransactionValidationError::TxNonceTooLow => EvmTxErrorWrapper::TxNonceTooLow,
			TransactionValidationError::TxNonceTooHigh => EvmTxErrorWrapper::TxNonceTooHigh,
			TransactionValidationError::InvalidFeeInput => EvmTxErrorWrapper::InvalidFeeInput,
			TransactionValidationError::InvalidChainId => EvmTxErrorWrapper::InvalidChainId,
			TransactionValidationError::InvalidSignature => EvmTxErrorWrapper::InvalidSignature,
			TransactionValidationError::UnknownError => EvmTxErrorWrapper::UnknownError,
		}
	}
}

// TODO: Reuse the frontier implementation
fn proof_size_base_cost(transaction: &Transaction) -> u64 {
	transaction
		.encode()
		.len()
		// pallet index
		.saturating_add(1)
		// call index
		.saturating_add(1) as u64
}
