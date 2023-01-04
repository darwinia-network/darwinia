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

extern crate alloc;

// std
use alloc::format;
use core::marker::PhantomData;
// frontier
use fp_evm::{ExitError, PrecompileFailure};
// moonbeam
use frame_support::{
	codec::{Decode, DecodeLimit as _},
	dispatch::{DispatchClass, Dispatchable, GetDispatchInfo, Pays, PostDispatchInfo},
	traits::{ConstU32, Get},
};
use pallet_evm::{AddressMapping, GasWeightMapping};
use precompile_utils::prelude::*;

// `DecodeLimit` specifies the max depth a call can use when decoding, as unbounded depth
// can be used to overflow the stack.
// Default value is 8, which is the same as in XCM call decoding.
pub struct Dispatch<T, DecodeLimit = ConstU32<8>> {
	_marker: PhantomData<(T, DecodeLimit)>,
}

#[precompile_utils::precompile]
impl<T, DecodeLimit> Dispatch<T, DecodeLimit>
where
	T: pallet_evm::Config,
	T::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + Decode,
	<T::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<T::AccountId>>,
	DecodeLimit: Get<u32>,
{
	#[precompile::public("execute(bytes)")]
	fn execute(
		handle: &mut impl PrecompileHandle,
		encoded_call: UnboundedBytes,
	) -> EvmResult<bool> {
		let target_gas = handle.gas_limit();
		let context = handle.context();

		let call = T::RuntimeCall::decode_with_depth_limit(
			DecodeLimit::get(),
			&mut encoded_call.as_bytes(),
		)
		.map_err(|_| revert("decode failed"))?;
		let info = call.get_dispatch_info();

		let valid_call = info.pays_fee == Pays::Yes && info.class == DispatchClass::Normal;
		if !valid_call {
			return Err(revert("invalid call"));
		}

		if let Some(gas) = target_gas {
			let valid_weight =
				info.weight.ref_time() <= T::GasWeightMapping::gas_to_weight(gas, false).ref_time();
			if !valid_weight {
				return Err(PrecompileFailure::Error { exit_status: ExitError::OutOfGas });
			}
		}

		let origin = T::AddressMapping::into_account_id(context.caller);
		match call.dispatch(Some(origin).into()) {
			Ok(post_info) => {
				let cost = T::GasWeightMapping::weight_to_gas(
					post_info.actual_weight.unwrap_or(info.weight),
				);

				handle.record_cost(cost)?;
				Ok(true)
			},
			Err(e) => Err(revert(format!("dispatch failed: {}", <&'static str>::from(e)))),
		}
	}
}
