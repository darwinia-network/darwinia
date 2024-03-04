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

// darwinia
use crate::*;

frame_support::parameter_types! {
	pub UnvestedFundsAllowedWithdrawReasons: frame_support::traits::WithdrawReasons =
		frame_support::traits::WithdrawReasons::except(
			frame_support::traits::WithdrawReasons::TRANSFER | frame_support::traits::WithdrawReasons::RESERVE
		);
}

impl pallet_vesting::Config for Runtime {
	type BlockNumberToBalance = sp_runtime::traits::ConvertInto;
	type Currency = Balances;
	type MinVestedTransfer = ConstU128<UNIT>;
	type RuntimeEvent = RuntimeEvent;
	type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
	type WeightInfo = weights::pallet_vesting::WeightInfo<Self>;

	// `VestingInfo` encode length is 36bytes. 28 schedules gets encoded as 1009 bytes, which is the
	// highest number of schedules that encodes less than 2^10.
	const MAX_VESTING_SCHEDULES: u32 = 28;
}
