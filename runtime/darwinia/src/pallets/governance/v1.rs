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
use super::*;

const ENACTMENT_PERIOD: u32 = 28 * DAYS;

impl pallet_democracy::Config for Runtime {
	type BlacklistOrigin = RootOrAtLeastTwoThird<TechnicalCollective>;
	type CancelProposalOrigin = RootOrAtLeastTwoThird<TechnicalCollective>;
	type CancellationOrigin = RootOrAtLeastTwoThird<TechnicalCollective>;
	type CooloffPeriod = ConstU32<{ 7 * DAYS }>;
	type Currency = Balances;
	type EnactmentPeriod = ConstU32<ENACTMENT_PERIOD>;
	// There are no plans to use this yet.
	type ExternalDefaultOrigin = Root;
	type ExternalMajorityOrigin = RootOrAtLeastHalf<CouncilCollective>;
	type ExternalOrigin = RootOrAtLeastHalf<CouncilCollective>;
	type FastTrackOrigin = RootOrAtLeastTwoThird<TechnicalCollective>;
	type FastTrackVotingPeriod = ConstU32<{ 3 * HOURS }>;
	type InstantAllowed = ConstBool<true>;
	type InstantOrigin = RootOrAll<TechnicalCollective>;
	type LaunchPeriod = ConstU32<{ 28 * DAYS }>;
	type MaxBlacklisted = ConstU32<100>;
	type MaxDeposits = ConstU32<100>;
	type MaxProposals = ConstU32<100>;
	type MaxVotes = ConstU32<100>;
	type MinimumDeposit = ConstU128<DARWINIA_PROPOSAL_REQUIREMENT>;
	type PalletsOrigin = OriginCaller;
	type Preimages = Preimage;
	type RuntimeEvent = RuntimeEvent;
	type Scheduler = Scheduler;
	type Slash = Treasury;
	type SubmitOrigin = frame_system::EnsureSigned<AccountId>;
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
	type VoteLockingPeriod = ConstU32<ENACTMENT_PERIOD>;
	type VotingPeriod = ConstU32<{ 28 * DAYS }>;
	type WeightInfo = weights::pallet_democracy::WeightInfo<Self>;
}
