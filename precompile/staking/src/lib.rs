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
#![deny(unused_crate_dependencies)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

// core
use core::marker::PhantomData;
// darwinia
use dp_staking::Stake;
// moonbeam
use precompile_utils::prelude::*;
// substrate
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	traits::OriginTrait,
};
use sp_core::{H160, U256};
use sp_runtime::{traits::Dispatchable, Perbill};
use sp_std::prelude::*;

pub struct Staking<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> Staking<Runtime>
where
	Runtime: dp_staking::Config + pallet_evm::Config,
	Runtime::RuntimeCall: GetDispatchInfo
		+ Dispatchable<PostInfo = PostDispatchInfo>
		+ From<dp_staking::Call<Runtime>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::AccountId: From<H160>,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin: OriginTrait,
	<<Runtime as dp_staking::Config>::Deposit as Stake>::Item: From<u16>,
{
	#[precompile::public("stake(uint256,uint256,uint16[])")]
	fn stake(
		handle: &mut impl PrecompileHandle,
		ring_amount: U256,
		kton_amount: U256,
		deposits: Vec<u16>,
	) -> EvmResult<bool> {
		let origin = handle.context().caller.into();
		let deposits = deposits.into_iter().map(|i| i.into()).collect();

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			dp_staking::Call::<Runtime>::stake {
				ring_amount: ring_amount.as_u128(),
				kton_amount: kton_amount.as_u128(),
				deposits,
			},
		)?;
		Ok(true)
	}

	#[precompile::public("unstake(uint256,uint256,uint16[])")]
	fn unstake(
		handle: &mut impl PrecompileHandle,
		ring_amount: U256,
		kton_amount: U256,
		deposits: Vec<u16>,
	) -> EvmResult<bool> {
		let origin = handle.context().caller.into();
		let deposits = deposits.into_iter().map(|i| i.into()).collect();

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			dp_staking::Call::<Runtime>::unstake {
				ring_amount: ring_amount.as_u128(),
				kton_amount: kton_amount.as_u128(),
				deposits,
			},
		)?;
		Ok(true)
	}

	#[precompile::public("restake(uint256,uint16[])")]
	fn restake(
		handle: &mut impl PrecompileHandle,
		ring_amount: U256,
		deposits: Vec<u16>,
	) -> EvmResult<bool> {
		let origin = handle.context().caller.into();
		let deposits = deposits.into_iter().map(|i| i.into()).collect();

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			dp_staking::Call::<Runtime>::restake { ring_amount: ring_amount.as_u128(), deposits },
		)?;
		Ok(true)
	}

	#[precompile::public("claim()")]
	fn claim(handle: &mut impl PrecompileHandle) -> EvmResult<bool> {
		let origin = handle.context().caller.into();

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			dp_staking::Call::<Runtime>::claim {},
		)?;
		Ok(true)
	}

	#[precompile::public("collect(uint32)")]
	fn collect(handle: &mut impl PrecompileHandle, commission: u32) -> EvmResult<bool> {
		let origin = handle.context().caller.into();

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			dp_staking::Call::<Runtime>::collect { commission: Perbill::from_percent(commission) },
		)?;
		Ok(true)
	}

	#[precompile::public("nominate(address)")]
	fn nominate(handle: &mut impl PrecompileHandle, target: Address) -> EvmResult<bool> {
		let target: H160 = target.into();
		let origin = handle.context().caller.into();

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			dp_staking::Call::<Runtime>::nominate { target: target.into() },
		)?;
		Ok(true)
	}

	#[precompile::public("chill()")]
	fn chill(handle: &mut impl PrecompileHandle) -> EvmResult<bool> {
		let origin = handle.context().caller.into();

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			dp_staking::Call::<Runtime>::chill {},
		)?;
		Ok(true)
	}

	#[precompile::public("payout(address)")]
	fn payout(handle: &mut impl PrecompileHandle, who: Address) -> EvmResult<bool> {
		let who: H160 = who.into();
		let origin = handle.context().caller.into();

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			dp_staking::Call::<Runtime>::payout { who: who.into() },
		)?;
		Ok(true)
	}
}
