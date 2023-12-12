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

pub use pallet_fee_market::Instance1 as WithPangoroFeeMarket;

// darwinia
use crate::*;

pub struct FeeMarketSlasher;

impl<T: pallet_fee_market::Config<I>, I: 'static> pallet_fee_market::Slasher<T, I>
	for FeeMarketSlasher
{
	fn calc_amount(
		locked_collateral: pallet_fee_market::BalanceOf<T, I>,
		timeout: frame_system::pallet_prelude::BlockNumberFor<T>,
	) -> pallet_fee_market::BalanceOf<T, I> {
		// substrate
		use sp_runtime::traits::UniqueSaturatedInto;

		let slash_each_block = 2 * UNIT;
		let slash_value =
			sp_runtime::traits::UniqueSaturatedInto::<Balance>::unique_saturated_into(timeout)
				.saturating_mul(
					sp_runtime::traits::UniqueSaturatedInto::<Balance>::unique_saturated_into(
						slash_each_block,
					),
				)
				.unique_saturated_into();

		core::cmp::min(locked_collateral, slash_value)
	}
}

frame_support::parameter_types! {
	pub const DutyRelayersRewardRatio: sp_runtime::Permill = sp_runtime::Permill::from_percent(60);
	pub const MessageRelayersRewardRatio: sp_runtime::Permill = sp_runtime::Permill::from_percent(80);
	pub const ConfirmRelayersRewardRatio: sp_runtime::Permill = sp_runtime::Permill::from_percent(20);
	pub const AssignedRelayerSlashRatio: sp_runtime::Permill = sp_runtime::Permill::from_percent(20);
}

impl pallet_fee_market::Config<WithPangoroFeeMarket> for Runtime {
	type AssignedRelayerSlashRatio = AssignedRelayerSlashRatio;
	type CollateralPerOrder = ConstU128<{ 50 * UNIT }>;
	type ConfirmRelayersRewardRatio = ConfirmRelayersRewardRatio;
	type Currency = Balances;
	type DutyRelayersRewardRatio = DutyRelayersRewardRatio;
	type LockId = pallet_config::FeeMarketLid;
	type MessageRelayersRewardRatio = MessageRelayersRewardRatio;
	type MinimumRelayFee = ConstU128<{ 15 * UNIT }>;
	type RuntimeEvent = RuntimeEvent;
	type Slasher = FeeMarketSlasher;
	type Slot = ConstU32<600>;
	type TreasuryPalletId = pallet_config::TreasuryPid;
	type WeightInfo = weights::pallet_fee_market::WeightInfo<Self>;
}
