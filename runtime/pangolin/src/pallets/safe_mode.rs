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

// darwinia
use crate::*;

/// Calls that can bypass the safe-mode pallet.
pub struct SafeModeWhitelistedCalls;
impl frame_support::traits::Contains<RuntimeCall> for SafeModeWhitelistedCalls {
	fn contains(call: &RuntimeCall) -> bool {
		match call {
			RuntimeCall::System(_) | RuntimeCall::SafeMode(_) | RuntimeCall::TxPause(_) => true,
			_ => false,
		}
	}
}

frame_support::parameter_types! {
	pub const EnterDuration: BlockNumber = 4 * HOURS;
	pub const EnterDepositAmount: Balance = 2_000_000 * UNIT;
	pub const ExtendDuration: BlockNumber = 2 * HOURS;
	pub const ExtendDepositAmount: Balance = 1_000_000 * UNIT;
	pub const ReleaseDelay: u32 = 2 * DAYS;
}

use frame_system::{EnsureRoot, EnsureRootWithSuccess};
impl pallet_safe_mode::Config for Runtime {
	type Currency = Balances;
	type EnterDepositAmount = EnterDepositAmount;
	type EnterDuration = EnterDuration;
	type ExtendDepositAmount = ExtendDepositAmount;
	type ExtendDuration = ExtendDuration;
	type ForceDepositOrigin = EnsureRootWithSuccess<AccountId, ConstU32<11>>;
	type ForceEnterOrigin = EnsureRootWithSuccess<AccountId, ConstU32<9>>;
	type ForceExitOrigin = EnsureRoot<AccountId>;
	type ForceExtendOrigin = EnsureRootWithSuccess<AccountId, ConstU32<11>>;
	type Notify = ();
	type ReleaseDelay = ReleaseDelay;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type WeightInfo = ();
	type WhitelistedCalls = SafeModeWhitelistedCalls;
}
