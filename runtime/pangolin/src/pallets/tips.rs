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

frame_support::parameter_types! {
	pub const TipFindersFee: sp_runtime::Percent = sp_runtime::Percent::from_percent(20);
}

impl pallet_tips::Config for Runtime {
	type DataDepositPerByte = ConstU128<{ darwinia_deposit(0, 1) }>;
	type MaximumReasonLength = ConstU32<16384>;
	type RuntimeEvent = RuntimeEvent;
	type TipCountdown = ConstU32<{ 10 * MINUTES }>;
	type TipFindersFee = TipFindersFee;
	type TipReportDepositBase = ConstU128<{ 100 * UNIT }>;
	type Tippers = PhragmenElection;
	type WeightInfo = weights::pallet_tips::WeightInfo<Self>;
}
