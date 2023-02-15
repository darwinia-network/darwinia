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

const MAX_CANDIDATES: u32 = 30;

frame_support::parameter_types! {
	pub const PhragmenElectionPalletId: frame_support::traits::LockIdentifier = *b"phrelect";
}

impl pallet_elections_phragmen::Config for Runtime {
	type CandidacyBond = ConstU128<{ 100 * MILLIUNIT }>;
	type ChangeMembers = Council;
	type Currency = Balances;
	type CurrencyToVote = frame_support::traits::U128CurrencyToVote;
	type DesiredMembers = ConstU32<COLLECTIVE_DESIRED_MEMBERS>;
	type DesiredRunnersUp = ConstU32<7>;
	type InitializeMembers = Council;
	type KickedMember = Treasury;
	type LoserCandidate = Treasury;
	type MaxCandidates = ConstU32<MAX_CANDIDATES>;
	type MaxVoters = ConstU32<{ 10 * MAX_CANDIDATES }>;
	type PalletId = PhragmenElectionPalletId;
	type RuntimeEvent = RuntimeEvent;
	// Daily council elections.
	type TermDuration = ConstU32<{ 7 * DAYS }>;
	// 1 storage item created, key size is 32 bytes, value size is 16+16.
	type VotingBondBase = ConstU128<{ darwinia_deposit(1, 64) }>;
	// Additional data per vote is 32 bytes (account id).
	type VotingBondFactor = ConstU128<{ darwinia_deposit(0, 32) }>;
	type WeightInfo = ();
}
