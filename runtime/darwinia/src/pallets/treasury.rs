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
	pub const ProposalBond: sp_runtime::Permill = sp_runtime::Permill::from_percent(5);
}

// In order to use `Tips`, which bounded by `pallet_treasury::Config` rather
// `pallet_treasury::Config<I>` Still use `DefaultInstance` here instead `Instance1`
impl pallet_treasury::Config for Runtime {
	type ApproveOrigin = Root;
	type Burn = ();
	type BurnDestination = ();
	type Currency = Balances;
	type MaxApprovals = ConstU32<100>;
	type OnSlash = Treasury;
	type PalletId = pallet_config::TreasuryPid;
	type ProposalBond = ProposalBond;
	type ProposalBondMaximum = ();
	type ProposalBondMinimum = ConstU128<DARWINIA_PROPOSAL_REQUIREMENT>;
	type RejectOrigin = RootOrAll<CouncilCollective>;
	type RuntimeEvent = RuntimeEvent;
	type SpendFunds = ();
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<Balance>;
	type SpendPeriod = ConstU32<{ 24 * DAYS }>;
	type WeightInfo = weights::pallet_treasury::WeightInfo<Self>;
}
