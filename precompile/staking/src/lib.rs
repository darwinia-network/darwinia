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
use sp_std::prelude::*;

pub struct Staking<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> Staking<Runtime>
where
	Runtime: darwinia_staking::Config + pallet_evm::Config,
	Runtime::RuntimeCall: GetDispatchInfo
		+ Dispatchable<PostInfo = PostDispatchInfo>
		+ From<darwinia_staking::Call<Runtime>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::AccountId: From<H160>,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin: OriginTrait,
{
	// #[precompile::public("unstake(uint256,uint16[])")]
	// fn unstake(
	// 	handle: &mut impl PrecompileHandle,
	// 	ring_amount: U256,
	// 	deposits: Vec<u16>,
	// ) -> EvmResult<bool> {
	// 	let origin = handle.context().caller.into();
	// 	let deposits = deposits.into_iter().map(|i| i.into()).collect();

	// 	RuntimeHelper::<Runtime>::try_dispatch(
	// 		handle,
	// 		Some(origin).into(),
	// 		darwinia_staking::Call::<Runtime>::unstake {
	// 			ring_amount: ring_amount.as_u128(),
	// 			deposits,
	// 		},
	// 	)?;
	// 	Ok(true)
	// }

	// #[precompile::public("payout(address)")]
	// fn payout(handle: &mut impl PrecompileHandle, who: Address) -> EvmResult<bool> {
	// 	let who: H160 = who.into();
	// 	let origin = handle.context().caller.into();

	// 	RuntimeHelper::<Runtime>::try_dispatch(
	// 		handle,
	// 		Some(origin).into(),
	// 		darwinia_staking::Call::<Runtime>::payout { who: who.into() },
	// 	)?;
	// 	Ok(true)
	// }
}
