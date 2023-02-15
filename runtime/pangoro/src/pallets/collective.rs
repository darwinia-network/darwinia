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

pub use pallet_collective::{Instance1 as CouncilCollective, Instance2 as TechnicalCollective};

// darwinia
use crate::*;

pub const COLLECTIVE_DESIRED_MEMBERS: u32 = 7;
pub const COLLECTIVE_MAX_MEMBERS: u32 = 100;
pub const COLLECTIVE_MAX_PROPOSALS: u32 = 100;

// Make sure that there are no more than `COLLECTIVE_MAX_MEMBERS` members elected via phragmen.
static_assertions::const_assert!(COLLECTIVE_DESIRED_MEMBERS <= COLLECTIVE_MAX_MEMBERS);

impl pallet_collective::Config<CouncilCollective> for Runtime {
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type MaxMembers = ConstU32<COLLECTIVE_MAX_MEMBERS>;
	type MaxProposals = ConstU32<100>;
	type MotionDuration = ConstU32<{ 3 * DAYS }>;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type WeightInfo = ();
}
impl pallet_collective::Config<TechnicalCollective> for Runtime {
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type MaxMembers = ConstU32<COLLECTIVE_MAX_MEMBERS>;
	type MaxProposals = ConstU32<100>;
	type MotionDuration = ConstU32<{ 3 * DAYS }>;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type WeightInfo = ();
}
