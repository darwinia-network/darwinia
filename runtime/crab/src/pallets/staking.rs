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

darwinia_common_runtime::fast_runtime_or_not!(
	SESSIONS_PER_DAY,
	Balance,
	// 5 minutes.
	24 * 60 / 5,
	// 6 hours.
	24 / 6
);

impl darwinia_staking::Config for Runtime {
	type KtonStaking = darwinia_staking::KtonStaking<Self>;
	type RewardPerSession = ConstU128<{ 80_000_000 * UNIT / 365 / SESSIONS_PER_DAY }>;
	type RingStaking = darwinia_staking::RingStaking<Self>;
	type RuntimeEvent = RuntimeEvent;
	type Treasury = pallet_config::TreasuryAccount;
	type WeightInfo = weights::darwinia_staking::WeightInfo<Self>;
}
