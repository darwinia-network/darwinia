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

// std
use core::marker::PhantomData;
// substrate
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	sp_runtime::traits::{Dispatchable, StaticLookup},
	traits::{
		fungibles::{
			approvals::Inspect as ApprovalInspect, metadata::Inspect as MetadataInspect, Inspect,
		},
		OriginTrait,
	},
};
use sp_core::{MaxEncodedLen, H160, U256};
use sp_runtime::traits::Bounded;
use sp_std::convert::{TryFrom, TryInto};
// moonbeam
use precompile_utils::prelude::*;

/// Solidity selector of the Transfer log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_TRANSFER: [u8; 32] = keccak256!("Transfer(address,address,uint256)");

/// Solidity selector of the Approval log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_APPROVAL: [u8; 32] = keccak256!("Approval(address,address,uint256)");

/// Balance of the specific assets.
type BalanceOf<R> = <R as pallet_assets::Config>::Balance;

/// AssetId of the specific assets.
type AssetIdOf<R> = <R as pallet_assets::Config>::AssetId;

/// AccountId of the runtime.
type AccountIdOf<R> = <R as frame_system::pallet::Config>::AccountId;

/// Convert from precompile AccountId to AssetId
///
/// Note: The AssetId generation must follow our precompile AccountId rule.
pub trait AccountToAssetId<AccountId, AssetId> {
	/// Get asset id from account id
	fn account_to_asset_id(account_id: AccountId) -> AssetId;
}

pub struct ERC20Assets<Runtime, AssetIdConverter>(PhantomData<(Runtime, AssetIdConverter)>);

#[precompile_utils::precompile]
impl<Runtime, AssetIdConverter> ERC20Assets<Runtime, AssetIdConverter>
where
	Runtime: pallet_assets::Config + pallet_evm::Config + frame_system::Config,
	AssetIdConverter: AccountToAssetId<Runtime::AccountId, AssetIdOf<Runtime>>,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<pallet_assets::Call<Runtime>>,
	AccountIdOf<Runtime>: From<H160>,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin: OriginTrait,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + solidity::Codec,
{
	#[precompile::public("totalSupply()")]
	#[precompile::view]
	fn total_supply(handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
		// Record proof_size cost for total_issuance
		handle.record_db_read::<Runtime>(pallet_assets::AssetDetails::<
			BalanceOf<Runtime>,
			AccountIdOf<Runtime>,
			BalanceOf<Runtime>,
		>::max_encoded_len())?;

		let asset_id = Self::asset_id(handle)?;
		Ok(pallet_assets::Pallet::<Runtime>::total_issuance(asset_id).into())
	}

	#[precompile::public("balanceOf(address)")]
	#[precompile::view]
	fn balance_of(handle: &mut impl PrecompileHandle, who: Address) -> EvmResult<U256> {
		// Record proof_size cost for the balance
		handle.record_db_read::<Runtime>(pallet_assets::AssetAccount::<
			BalanceOf<Runtime>,
			AccountIdOf<Runtime>,
			(),
			BalanceOf<Runtime>,
		>::max_encoded_len())?;

		let asset_id = Self::asset_id(handle)?;
		let who: H160 = who.into();
		let amount: U256 = {
			let who: AccountIdOf<Runtime> = who.into();
			pallet_assets::Pallet::<Runtime>::balance(asset_id, &who).into()
		};

		Ok(amount)
	}

	#[precompile::public("allowance(address,address)")]
	#[precompile::view]
	fn allowance(
		handle: &mut impl PrecompileHandle,
		owner: Address,
		spender: Address,
	) -> EvmResult<U256> {
		// Record proof_size cost for the allowance
		handle.record_db_read::<Runtime>(pallet_assets::Approval::<
			BalanceOf<Runtime>,
			BalanceOf<Runtime>,
		>::max_encoded_len())?;

		let owner: H160 = owner.into();
		let spender: H160 = spender.into();
		let asset_id = Self::asset_id(handle)?;
		let amount: U256 = {
			let owner: AccountIdOf<Runtime> = owner.into();
			let spender: AccountIdOf<Runtime> = spender.into();
			pallet_assets::Pallet::<Runtime>::allowance(asset_id, &owner, &spender).into()
		};

		Ok(amount)
	}

	#[precompile::public("approve(address,uint256)")]
	fn approve(
		handle: &mut impl PrecompileHandle,
		spender: Address,
		value: U256,
	) -> EvmResult<bool> {
		// Record proof_size cost for the allowance
		handle.record_db_read::<Runtime>(pallet_assets::Approval::<
			BalanceOf<Runtime>,
			BalanceOf<Runtime>,
		>::max_encoded_len())?;
		handle.record_log_costs_manual(3, 32)?;

		let spender: H160 = spender.into();
		let asset_id = Self::asset_id(handle)?;
		{
			let owner: AccountIdOf<Runtime> = handle.context().caller.into();
			let spender: AccountIdOf<Runtime> = spender.into();
			// Amount saturate if too high.
			let amount = value.try_into().unwrap_or_else(|_| Bounded::max_value());

			// If previous approval exists, we need to clean it
			if pallet_assets::Pallet::<Runtime>::allowance(asset_id.clone(), &owner, &spender)
				!= 0u32.into()
			{
				RuntimeHelper::<Runtime>::try_dispatch(
					handle,
					Some(owner.clone()).into(),
					pallet_assets::Call::<Runtime>::cancel_approval {
						id: asset_id.clone().into(),
						delegate: Runtime::Lookup::unlookup(spender.clone()),
					},
				)?;
			}
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(owner).into(),
				pallet_assets::Call::<Runtime>::approve_transfer {
					id: asset_id.into(),
					delegate: Runtime::Lookup::unlookup(spender),
					amount,
				},
			)?;
		}

		log3(
			handle.context().address,
			SELECTOR_LOG_APPROVAL,
			handle.context().caller,
			spender,
			solidity::encode_event_data(value),
		)
		.record(handle)?;

		Ok(true)
	}

	#[precompile::public("transfer(address,uint256)")]
	fn transfer(handle: &mut impl PrecompileHandle, to: Address, value: U256) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;
		let asset_id = Self::asset_id(handle)?;

		let to: H160 = to.into();
		let value = Self::u256_to_amount(value).in_field("value")?;
		{
			let origin: AccountIdOf<Runtime> = handle.context().caller.into();
			let to: AccountIdOf<Runtime> = to.into();

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_assets::Call::<Runtime>::transfer {
					id: asset_id.into(),
					target: Runtime::Lookup::unlookup(to),
					amount: value,
				},
			)?;
		}

		log3(
			handle.context().address,
			SELECTOR_LOG_TRANSFER,
			handle.context().caller,
			to,
			solidity::encode_event_data(value),
		)
		.record(handle)?;

		Ok(true)
	}

	#[precompile::public("transferFrom(address,address,uint256)")]
	fn transfer_from(
		handle: &mut impl PrecompileHandle,
		from: Address,
		to: Address,
		value: U256,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;
		let asset_id = Self::asset_id(handle)?;

		let from: H160 = from.into();
		let to: H160 = to.into();
		let value = Self::u256_to_amount(value).in_field("value")?;
		{
			let caller: AccountIdOf<Runtime> = handle.context().caller.into();
			let from: AccountIdOf<Runtime> = from.into();
			let to: AccountIdOf<Runtime> = to.into();

			// If caller is "from", it can spend as much as it wants from its own balance.
			if caller != from {
				RuntimeHelper::<Runtime>::try_dispatch(
					handle,
					Some(caller).into(),
					pallet_assets::Call::<Runtime>::transfer_approved {
						id: asset_id.into(),
						owner: Runtime::Lookup::unlookup(from),
						destination: Runtime::Lookup::unlookup(to),
						amount: value,
					},
				)?;
			} else {
				RuntimeHelper::<Runtime>::try_dispatch(
					handle,
					Some(from).into(),
					pallet_assets::Call::<Runtime>::transfer {
						id: asset_id.into(),
						target: Runtime::Lookup::unlookup(to),
						amount: value,
					},
				)?;
			}
		}

		log3(
			handle.context().address,
			SELECTOR_LOG_TRANSFER,
			from,
			to,
			solidity::encode_event_data(value),
		)
		.record(handle)?;

		Ok(true)
	}

	#[precompile::public("name()")]
	#[precompile::view]
	fn name(handle: &mut impl PrecompileHandle) -> EvmResult<UnboundedBytes> {
		// Record proof_size cost for the asset metadata
		handle.record_db_read::<Runtime>(pallet_assets::AssetMetadata::<
			BalanceOf<Runtime>,
			[u8; 50], // 50 refers to the StringLimit of the pallet_assets
		>::max_encoded_len())?;

		let asset_id = Self::asset_id(handle)?;
		Ok(pallet_assets::Pallet::<Runtime>::name(asset_id).as_slice().into())
	}

	#[precompile::public("symbol()")]
	#[precompile::view]
	fn symbol(handle: &mut impl PrecompileHandle) -> EvmResult<UnboundedBytes> {
		// Record proof_size cost for the asset metadata
		handle.record_db_read::<Runtime>(pallet_assets::AssetMetadata::<
			BalanceOf<Runtime>,
			[u8; 50], // 50 refers to the StringLimit of the pallet_assets
		>::max_encoded_len())?;

		let asset_id = Self::asset_id(handle)?;
		Ok(pallet_assets::Pallet::<Runtime>::symbol(asset_id).as_slice().into())
	}

	#[precompile::public("decimals()")]
	#[precompile::view]
	fn decimals(handle: &mut impl PrecompileHandle) -> EvmResult<u8> {
		// Record proof_size cost for the asset metadata
		handle.record_db_read::<Runtime>(pallet_assets::AssetMetadata::<
			BalanceOf<Runtime>,
			[u8; 50], // 50 refers to the StringLimit of the pallet_assets
		>::max_encoded_len())?;

		let asset_id = Self::asset_id(handle)?;
		Ok(pallet_assets::Pallet::<Runtime>::decimals(asset_id))
	}

	#[precompile::public("mint(address,uint256)")]
	fn mint(handle: &mut impl PrecompileHandle, to: Address, value: U256) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;
		let asset_id = Self::asset_id(handle)?;

		let to: H160 = to.into();
		let value = Self::u256_to_amount(value).in_field("value")?;
		{
			let origin: AccountIdOf<Runtime> = handle.context().caller.into();
			let to: AccountIdOf<Runtime> = to.into();

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_assets::Call::<Runtime>::mint {
					id: asset_id.into(),
					beneficiary: Runtime::Lookup::unlookup(to),
					amount: value,
				},
			)?;
		}

		log3(
			handle.context().address,
			SELECTOR_LOG_TRANSFER,
			H160::default(),
			to,
			solidity::encode_event_data(value),
		)
		.record(handle)?;

		Ok(true)
	}

	#[precompile::public("burn(address,uint256)")]
	fn burn(handle: &mut impl PrecompileHandle, from: Address, value: U256) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;
		let asset_id = Self::asset_id(handle)?;

		let from: H160 = from.into();
		let value = Self::u256_to_amount(value).in_field("value")?;
		{
			let origin: AccountIdOf<Runtime> = handle.context().caller.into();
			let from: AccountIdOf<Runtime> = from.into();

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_assets::Call::<Runtime>::burn {
					id: asset_id.into(),
					who: Runtime::Lookup::unlookup(from),
					amount: value,
				},
			)?;
		}

		log3(
			handle.context().address,
			SELECTOR_LOG_TRANSFER,
			from,
			H160::default(),
			solidity::encode_event_data(value),
		)
		.record(handle)?;

		Ok(true)
	}

	#[precompile::public("transfer_ownership(address)")]
	fn transfer_ownership(handle: &mut impl PrecompileHandle, owner: Address) -> EvmResult<bool> {
		let asset_id = Self::asset_id(handle)?;

		let owner: H160 = owner.into();
		{
			let origin: AccountIdOf<Runtime> = handle.context().caller.into();
			let owner: AccountIdOf<Runtime> = owner.into();

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_assets::Call::<Runtime>::transfer_ownership {
					id: asset_id.into(),
					owner: Runtime::Lookup::unlookup(owner),
				},
			)?;
		}

		Ok(true)
	}

	#[precompile::public("freeze(address)")]
	fn freeze(handle: &mut impl PrecompileHandle, account: Address) -> EvmResult<bool> {
		let asset_id = Self::asset_id(handle)?;

		let account: H160 = account.into();
		{
			let origin: AccountIdOf<Runtime> = handle.context().caller.into();
			let account: AccountIdOf<Runtime> = account.into();

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_assets::Call::<Runtime>::freeze {
					id: asset_id.into(),
					who: Runtime::Lookup::unlookup(account),
				},
			)?;
		}

		Ok(true)
	}

	#[precompile::public("thaw(address)")]
	fn thaw(handle: &mut impl PrecompileHandle, account: Address) -> EvmResult<bool> {
		let asset_id = Self::asset_id(handle)?;

		let account: H160 = account.into();
		{
			let origin: AccountIdOf<Runtime> = handle.context().caller.into();
			let account: AccountIdOf<Runtime> = account.into();

			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_assets::Call::<Runtime>::thaw {
					id: asset_id.into(),
					who: Runtime::Lookup::unlookup(account),
				},
			)?;
		}

		Ok(true)
	}

	fn asset_id(handle: &mut impl PrecompileHandle) -> EvmResult<AssetIdOf<Runtime>> {
		// Record proof_size cost for the maybe_total_supply
		handle.record_db_read::<Runtime>(pallet_assets::AssetDetails::<
			BalanceOf<Runtime>,
			AccountIdOf<Runtime>,
			BalanceOf<Runtime>,
		>::max_encoded_len())?;

		let asset_id = AssetIdConverter::account_to_asset_id(handle.code_address().into());
		if pallet_assets::Pallet::<Runtime>::maybe_total_supply(asset_id.clone()).is_some() {
			return Ok(asset_id);
		}
		Err(revert("The asset not exist!"))
	}

	fn u256_to_amount(value: U256) -> MayRevert<BalanceOf<Runtime>> {
		value.try_into().map_err(|_| RevertReason::value_is_too_large("balance type").into())
	}
}
