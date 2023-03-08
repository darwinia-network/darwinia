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

const ENACTMENT_PERIOD: u32 = 8 * DAYS;

impl pallet_democracy::Config for Runtime {
	type BlacklistOrigin = Root;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = RootOrAll<TechnicalCollective>;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin = RootOrAtLeastTwoThird<CouncilCollective>;
	type CooloffPeriod = ConstU32<{ 7 * DAYS }>;
	type Currency = Balances;
	type EnactmentPeriod = ConstU32<ENACTMENT_PERIOD>;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin = RootOrAll<CouncilCollective>;
	/// A majority can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin = RootOrAtLeastHalf<CouncilCollective>;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin = RootOrAtLeastHalf<CouncilCollective>;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin = RootOrAtLeastTwoThird<TechnicalCollective>;
	type FastTrackVotingPeriod = ConstU32<{ 3 * HOURS }>;
	type InstantAllowed = ConstBool<true>;
	type InstantOrigin = RootOrAll<TechnicalCollective>;
	type LaunchPeriod = ConstU32<{ 7 * DAYS }>;
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
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cool-off period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
	type VoteLockingPeriod = ConstU32<ENACTMENT_PERIOD>;
	type VotingPeriod = ConstU32<{ 7 * DAYS }>;
	type WeightInfo = weights::pallet_democracy::WeightInfo<Self>;
}
