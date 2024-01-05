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

mod origin;
use origin::*;
pub use origin::{custom_origins, GeneralAdmin};

mod track;
use track::*;

mod v1;

pub use pallet_collective::{Instance1 as CouncilCollective, Instance2 as TechnicalCollective};

pub(super) use crate::*;

pub const COLLECTIVE_DESIRED_MEMBERS: u32 = 7;
pub const COLLECTIVE_MAX_MEMBERS: u32 = 100;
pub const COLLECTIVE_MAX_PROPOSALS: u32 = 100;

// Make sure that there are no more than `COLLECTIVE_MAX_MEMBERS` members elected via phragmen.
static_assertions::const_assert!(COLLECTIVE_DESIRED_MEMBERS <= COLLECTIVE_MAX_MEMBERS);

frame_support::parameter_types! {
	pub MaxProposalWeight: frame_support::weights::Weight = sp_runtime::Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
}

impl pallet_collective::Config<CouncilCollective> for Runtime {
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type MaxMembers = ConstU32<COLLECTIVE_MAX_MEMBERS>;
	type MaxProposalWeight = MaxProposalWeight;
	type MaxProposals = ConstU32<100>;
	type MotionDuration = ConstU32<{ 3 * DAYS }>;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SetMembersOrigin = RootOr<GeneralAdmin>;
	type WeightInfo = weights::pallet_collective_council::WeightInfo<Self>;
}
impl pallet_collective::Config<TechnicalCollective> for Runtime {
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type MaxMembers = ConstU32<COLLECTIVE_MAX_MEMBERS>;
	type MaxProposalWeight = MaxProposalWeight;
	type MaxProposals = ConstU32<100>;
	type MotionDuration = ConstU32<{ 3 * DAYS }>;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SetMembersOrigin = RootOr<GeneralAdmin>;
	type WeightInfo = weights::pallet_collective_technical_committee::WeightInfo<Self>;
}

impl pallet_conviction_voting::Config for Runtime {
	type Currency = Balances;
	type MaxTurnout = frame_support::traits::TotalIssuanceOf<Balances, Self::AccountId>;
	type MaxVotes = ConstU32<512>;
	type Polls = Referenda;
	type RuntimeEvent = RuntimeEvent;
	type VoteLockingPeriod = ConstU32<{ DAYS }>;
	// type WeightInfo = weights::pallet_conviction_voting::WeightInfo<Self>;
	type WeightInfo = ();
}

pallet_referenda::impl_tracksinfo_get!(TracksInfo, Balance, BlockNumber);

impl pallet_referenda::Config for Runtime {
	type AlarmInterval = ConstU32<1>;
	type CancelOrigin = RootOr<ReferendumCanceller>;
	type Currency = Balances;
	type KillOrigin = RootOr<ReferendumKiller>;
	type MaxQueued = ConstU32<100>;
	type Preimages = Preimage;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type Scheduler = Scheduler;
	type Slash = Treasury;
	type SubmissionDeposit = ConstU128<{ DARWINIA_PROPOSAL_REQUIREMENT }>;
	type SubmitOrigin = frame_system::EnsureSigned<AccountId>;
	type Tally = pallet_conviction_voting::TallyOf<Self>;
	type Tracks = TracksInfo;
	type UndecidingTimeout = ConstU32<{ 14 * DAYS }>;
	type Votes = pallet_conviction_voting::VotesOf<Self>;
	// type WeightInfo = weights::pallet_referenda::WeightInfo<Self>;
	type WeightInfo = ();
}

impl custom_origins::Config for Runtime {}

// The purpose of this pallet is to queue calls to be dispatched as by root later => the Dispatch
// origin corresponds to the Gov2 Whitelist track.
impl pallet_whitelist::Config for Runtime {
	type DispatchWhitelistedOrigin = RootOr<WhitelistedCaller>;
	type Preimages = Preimage;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	// type WeightInfo = weights::pallet_whitelist::WeightInfo<Self>;
	type WeightInfo = ();
	type WhitelistOrigin = RootOrAtLeastFourFifth<TechnicalCollective>;
}

frame_support::parameter_types! {
	pub const ProposalBond: sp_runtime::Permill = sp_runtime::Permill::from_percent(5);
}

impl pallet_treasury::Config for Runtime {
	type ApproveOrigin = RootOr<GeneralAdmin>;
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
	type SpendPeriod = ConstU32<{ 28 * DAYS }>;
	type WeightInfo = weights::pallet_treasury::WeightInfo<Self>;
}
