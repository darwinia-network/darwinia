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

mod origin;
use origin::*;
pub use origin::{custom_origins, GeneralAdmin};

mod track;
use track::*;

pub use pallet_collective::Instance2 as TechnicalCollective;

pub(super) use crate::*;

impl pallet_collective::Config<TechnicalCollective> for Runtime {
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type MaxMembers = ConstU32<100>;
	type MaxProposalWeight = pallet_config::MaxProposalWeight;
	type MaxProposals = ConstU32<100>;
	type MotionDuration = ConstU32<{ 3 * DAYS }>;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SetMembersOrigin = RootOr<GeneralAdmin>;
	type WeightInfo = weights::pallet_collective::WeightInfo<Self>;
}

impl pallet_conviction_voting::Config for Runtime {
	type Currency = Balances;
	type MaxTurnout = frame_support::traits::TotalIssuanceOf<Balances, Self::AccountId>;
	type MaxVotes = ConstU32<512>;
	type Polls = Referenda;
	type RuntimeEvent = RuntimeEvent;
	type VoteLockingPeriod = ConstU32<{ DAYS }>;
	type WeightInfo = weights::pallet_conviction_voting::WeightInfo<Self>;
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
	type SubmitOrigin = frame_system::EnsureSigned<Self::AccountId>;
	type Tally = pallet_conviction_voting::TallyOf<Self>;
	type Tracks = TracksInfo;
	type UndecidingTimeout = ConstU32<{ 14 * DAYS }>;
	type Votes = pallet_conviction_voting::VotesOf<Self>;
	type WeightInfo = weights::pallet_referenda::WeightInfo<Self>;
}

impl custom_origins::Config for Runtime {}

frame_support::ord_parameter_types! {
	// 0x663fC3000f0101BF16FDc9F73F02DA6Efa8c5875.
	pub const DispatchWhitelistedDao: AccountId = AccountId::from([
		102, 63, 195, 0, 15, 1, 1, 191, 22, 253, 201, 247, 63, 2, 218, 110, 250, 140, 88, 117,
	]);
}

// The purpose of this pallet is to queue calls to be dispatched as by root later => the Dispatch
// origin corresponds to the Gov2 Whitelist track.
impl pallet_whitelist::Config for Runtime {
	#[cfg(feature = "runtime-benchmarks")]
	type DispatchWhitelistedOrigin = Root;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type DispatchWhitelistedOrigin = RootOrDiverse<
		frame_support::traits::EitherOfDiverse<
			WhitelistedCaller,
			frame_system::EnsureSignedBy<DispatchWhitelistedDao, Self::AccountId>,
		>,
	>;
	type Preimages = Preimage;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_whitelist::WeightInfo<Self>;
	type WhitelistOrigin = RootOrAtLeastFourFifth<TechnicalCollective>;
}

impl pallet_treasury::Config for Runtime {
	type AssetKind = ();
	type BalanceConverter = frame_support::traits::tokens::UnityAssetBalanceConversion;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = benchmark_helper::Treasury;
	type Beneficiary = Self::AccountId;
	type BeneficiaryLookup = Self::Lookup;
	type Burn = ();
	type BurnDestination = ();
	type Currency = Balances;
	type MaxApprovals = ConstU32<100>;
	type PalletId = pallet_config::TreasuryPid;
	type Paymaster =
		frame_support::traits::tokens::PayFromAccount<Balances, pallet_config::TreasuryAccount>;
	type PayoutPeriod = ConstU32<{ 14 * DAYS }>;
	type RejectOrigin = RootOr<GeneralAdmin>;
	type RuntimeEvent = RuntimeEvent;
	type SpendFunds = ();
	type SpendOrigin = EitherOf<
		frame_system::EnsureRootWithSuccess<Self::AccountId, pallet_config::MaxBalance>,
		Spender,
	>;
	type SpendPeriod = ConstU32<{ 14 * DAYS }>;
	type WeightInfo = weights::pallet_treasury::WeightInfo<Self>;
}
