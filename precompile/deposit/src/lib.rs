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
mod tests;

// core
use core::marker::PhantomData;
// moonbeam
use precompile_utils::prelude::*;
// polkadot-sdk
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	traits::OriginTrait,
};
use sp_core::{H160, U256};
use sp_runtime::traits::Dispatchable;

/// AccountId of the runtime.
type AccountIdOf<R> = <R as frame_system::pallet::Config>::AccountId;

pub struct Deposit<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> Deposit<Runtime>
where
	Runtime: darwinia_deposit::Config + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<darwinia_deposit::Call<Runtime>>,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin: OriginTrait,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	AccountIdOf<Runtime>: From<H160>,
{
	#[precompile::public("lock(uint256,uint8)")]
	fn lock(handle: &mut impl PrecompileHandle, amount: U256, months: u8) -> EvmResult<bool> {
		let origin: AccountIdOf<Runtime> = handle.context().caller.into();

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			darwinia_deposit::Call::<Runtime>::lock { amount: amount.as_u128(), months },
		)?;

		Ok(true)
	}

	#[precompile::public("claim()")]
	fn claim(handle: &mut impl PrecompileHandle) -> EvmResult<bool> {
		let origin: AccountIdOf<Runtime> = handle.context().caller.into();

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			darwinia_deposit::Call::<Runtime>::claim {},
		)?;

		Ok(true)
	}

	#[precompile::public("claim_with_penalty(uint8)")]
	fn claim_with_penalty(handle: &mut impl PrecompileHandle, id: u8) -> EvmResult<bool> {
		let origin: AccountIdOf<Runtime> = handle.context().caller.into();

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			darwinia_deposit::Call::<Runtime>::claim_with_penalty { id: id.into() },
		)?;

		Ok(true)
	}

	#[precompile::public("migrate(address)")]
	fn migrate(handle: &mut impl PrecompileHandle, who: Address) -> EvmResult<bool> {
		let origin: AccountIdOf<Runtime> = handle.context().caller.into();
		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			darwinia_deposit::Call::<Runtime>::migrate { who: who.into() },
		)?;
		Ok(true)
	}
}
