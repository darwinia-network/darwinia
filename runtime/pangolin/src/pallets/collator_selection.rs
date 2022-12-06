// This file is part of Darwinia.
//
// Copyright (C) 2018-2022 Darwinia Network
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

// We allow root only to execute privileged collator selection operations.
pub type CollatorSelectionUpdateOrigin = EnsureRoot<AccountId>;
frame_support::parameter_types! {
	pub const PotId: PalletId = PalletId(*b"PotStake");
	pub const ExecutiveBody: BodyId = BodyId::Executive;
}

impl pallet_collator_selection::Config for Runtime {
	type Currency = Balances;
	// should be a multiple of session or things will get inconsistent
	type KickThreshold = Period;
	type MaxCandidates = ConstU32<1000>;
	type MaxInvulnerables = ConstU32<100>;
	type MinCandidates = ConstU32<5>;
	type PotId = PotId;
	type RuntimeEvent = RuntimeEvent;
	type UpdateOrigin = CollatorSelectionUpdateOrigin;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = weights::pallet_collator_selection::WeightInfo<Self>;
}
