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

darwinia_common_runtime::fast_runtime_or_not!(DURATION, BlockNumber, 5 * MINUTES, 14 * DAYS);

impl darwinia_staking::Config for Runtime {
	type Currency = Balances;
	type IssuingManager = darwinia_staking::TreasuryIssuing<Self, ConstU128<{ 20_000 * UNIT }>>;
	type KtonStaking = darwinia_staking::KtonStaking<Self>;
	type RingStaking = darwinia_staking::RingStaking<Self>;
	type RuntimeEvent = RuntimeEvent;
	type Treasury = pallet_config::TreasuryAccount;
	type UnixTime = Timestamp;
	type WeightInfo = weights::darwinia_staking::WeightInfo<Self>;
}
