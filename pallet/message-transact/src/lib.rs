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
#![cfg_attr(not(test), deny(unused_crate_dependencies))]

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
use fp_evm::{CheckEvmTransaction, CheckEvmTransactionConfig, InvalidEvmTransactionError};
use pallet_evm::{FeeCalculator, GasWeightMapping};
// substrate
use frame_support::{traits::EnsureOrigin, PalletError, RuntimeDebug};
use sp_core::{H160, U256};
use sp_std::boxed::Box;

pub use pallet::*;

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum LcmpEthOrigin {
	MessageTransact(H160),
}

pub fn ensure_message_transact<OuterOrigin>(o: OuterOrigin) -> Result<H160, &'static str>
where
	OuterOrigin: Into<Result<LcmpEthOrigin, OuterOrigin>>,
{
	match o.into() {
		Ok(LcmpEthOrigin::MessageTransact(n)) => Ok(n),
		_ => Err("bad origin: expected to be an Lcmp Ethereum transaction"),
	}
}

pub struct EnsureLcmpEthOrigin;
impl<O: Into<Result<LcmpEthOrigin, O>> + From<LcmpEthOrigin>> EnsureOrigin<O>
	for EnsureLcmpEthOrigin
{
	type Success = H160;

	fn try_origin(o: O) -> Result<Self::Success, O> {
		o.into().map(|o| match o {
			LcmpEthOrigin::MessageTransact(id) => id,
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> O {
		O::from(LcmpEthOrigin::MessageTransact(Default::default()))
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::origin]
	pub type Origin = LcmpEthOrigin;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		/// Handler for applying an already validated transaction
		type ValidatedTransaction: ValidatedTransaction;
		/// Origin for message transact
		type LcmpEthOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = H160>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// EVM validation errors.
		MessageTransactError(EvmTxErrorWrapper),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		OriginFor<T>: Into<Result<LcmpEthOrigin, OriginFor<T>>>,
	{
		/// This call can only be called by the lcmp message layer and is not available to normal
		/// users.
		#[pallet::call_index(0)]
		#[pallet::weight({
			let without_base_extrinsic_weight = true;
			<T as pallet_evm::Config>::GasWeightMapping::gas_to_weight({
				let transaction_data: TransactionData = (&**transaction).into();
				transaction_data.gas_limit.unique_saturated_into()
			}, without_base_extrinsic_weight)
		})]
		pub fn message_transact(
			origin: OriginFor<T>,
			mut transaction: Box<Transaction>,
		) -> DispatchResultWithPostInfo {
			let source = ensure_message_transact(origin)?;
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
			let _ = CheckEvmTransaction::<EvmTxErrorWrapper>::new(
				CheckEvmTransactionConfig {
					evm_config: T::config(),
					block_gas_limit: T::BlockGasLimit::get(),
					base_fee,
					chain_id: T::ChainId::get(),
					is_transactional: true,
				},
				transaction_data.into(),
			)
			.validate_in_block_for(&who)
			.and_then(|v| v.with_chain_id())
			.and_then(|v| v.with_base_fee())
			.and_then(|v| v.with_balance_for(&who))
			.map_err(|e| <Error<T>>::MessageTransactError(e))?;

			T::ValidatedTransaction::apply(source, *transaction)
		}
	}
}

#[derive(Encode, Decode, TypeInfo, PalletError)]
pub enum EvmTxErrorWrapper {
	GasLimitTooLow,
	GasLimitTooHigh,
	GasPriceTooLow,
	PriorityFeeTooHigh,
	BalanceTooLow,
	TxNonceTooLow,
	TxNonceTooHigh,
	InvalidPaymentInput,
	InvalidChainId,
}

impl From<InvalidEvmTransactionError> for EvmTxErrorWrapper {
	fn from(validation_error: InvalidEvmTransactionError) -> Self {
		match validation_error {
			InvalidEvmTransactionError::GasLimitTooLow => EvmTxErrorWrapper::GasLimitTooLow,
			InvalidEvmTransactionError::GasLimitTooHigh => EvmTxErrorWrapper::GasLimitTooHigh,
			InvalidEvmTransactionError::GasPriceTooLow => EvmTxErrorWrapper::GasPriceTooLow,
			InvalidEvmTransactionError::PriorityFeeTooHigh => EvmTxErrorWrapper::PriorityFeeTooHigh,
			InvalidEvmTransactionError::BalanceTooLow => EvmTxErrorWrapper::BalanceTooLow,
			InvalidEvmTransactionError::TxNonceTooLow => EvmTxErrorWrapper::TxNonceTooLow,
			InvalidEvmTransactionError::TxNonceTooHigh => EvmTxErrorWrapper::TxNonceTooHigh,
			InvalidEvmTransactionError::InvalidPaymentInput =>
				EvmTxErrorWrapper::InvalidPaymentInput,
			InvalidEvmTransactionError::InvalidChainId => EvmTxErrorWrapper::InvalidChainId,
		}
	}
}

/// Calculates the fee for a relayer to submit an LCMP EVM transaction.
///
/// The gas_price of an LCMP EVM transaction is always the min_gas_price(), which is a fixed value.
/// Therefore, only the gas_limit and value of the transaction should be considered in the
/// calculation of the fee, and the gas_price of the transaction itself can be ignored.
pub fn total_payment<T: pallet_evm::Config>(tx_data: TransactionData) -> U256 {
	let base_fee = <T as pallet_evm::Config>::FeeCalculator::min_gas_price().0;
	let fee = base_fee.saturating_mul(tx_data.gas_limit);

	tx_data.value.saturating_add(fee)
}
