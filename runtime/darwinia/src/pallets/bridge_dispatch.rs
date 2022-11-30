// This file is part of Darwinia.
//
// Copyright (C) 2018-2022 Darwinia Network
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

pub use pallet_bridge_dispatch::Instance1 as WithCrabDispatch;

// darwinia
use crate::*;
use bp_message_dispatch::{CallValidate, IntoDispatchOrigin as IntoDispatchOriginT};
use bp_messages::{LaneId, MessageNonce};
use darwinia_message_transact::{total_payment, Call as MessageTransactCall, LcmpEthOrigin};
use pallet_bridge_dispatch::Config;
// substrate
use frame_support::ensure;
use frame_system::RawOrigin;
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidityError};

pub struct CallValidator;
impl CallValidate<AccountId, RuntimeOrigin, RuntimeCall> for CallValidator {
	fn check_receiving_before_dispatch(
		relayer_account: &AccountId,
		call: &RuntimeCall,
	) -> Result<(), &'static str> {
		match call {
			RuntimeCall::MessageTransact(MessageTransactCall::message_transact {
				transaction: tx,
			}) => {
				let total_payment = total_payment::<Runtime>(tx.into());
				let relayer =
					pallet_evm::Pallet::<Runtime>::account_basic(&H160(relayer_account.0)).0;

				ensure!(relayer.balance >= total_payment, "Insufficient balance");
				Ok(())
			},
			_ => Ok(()),
		}
	}

	fn call_validate(
		relayer_account: &AccountId,
		origin: &RuntimeOrigin,
		call: &RuntimeCall,
	) -> Result<(), TransactionValidityError> {
		match call {
			RuntimeCall::MessageTransact(MessageTransactCall::message_transact {
				transaction: tx,
			}) => match origin.caller {
				OriginCaller::MessageTransact(LcmpEthOrigin::MessageTransact(id)) => {
					let total_payment = total_payment::<Runtime>(tx.into());
					pallet_balances::Pallet::<Runtime>::transfer(
						RawOrigin::Signed(*relayer_account).into(),
						id.into(),
						total_payment.as_u128(),
					)
					.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Payment))?;

					Ok(())
				},
				_ => Err(TransactionValidityError::Invalid(InvalidTransaction::BadSigner)),
			},
			_ => Ok(()),
		}
	}
}

pub struct IntoDispatchOrigin;
impl IntoDispatchOriginT<AccountId, RuntimeCall, RuntimeOrigin> for IntoDispatchOrigin {
	fn into_dispatch_origin(id: &AccountId, call: &RuntimeCall) -> RuntimeOrigin {
		match call {
			RuntimeCall::MessageTransact(darwinia_message_transact::Call::message_transact {
				..
			}) => darwinia_message_transact::LcmpEthOrigin::MessageTransact(H160(id.0)).into(),
			_ => frame_system::RawOrigin::Signed(id.clone()).into(),
		}
	}
}

impl Config<WithCrabDispatch> for Runtime {
	type AccountIdConverter = bp_darwinia::AccountIdConverter;
	type BridgeMessageId = (LaneId, MessageNonce);
	type CallValidator = CallValidator;
	type EncodedCall = bm_crab::FromCrabEncodedCall;
	type IntoDispatchOrigin = IntoDispatchOrigin;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type SourceChainAccountId = bp_crab::AccountId;
	type TargetChainAccountPublic = bp_darwinia::AccountPublic;
	type TargetChainSignature = bp_darwinia::Signature;
}
