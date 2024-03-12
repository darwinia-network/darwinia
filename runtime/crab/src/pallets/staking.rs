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
// substrate
use frame_support::traits::Currency;

fast_runtime_or_not!(DURATION, BlockNumber, 5 * MINUTES, 14 * DAYS);

type MinStakingDuration = ConstU32<{ DURATION }>;

pub enum RingStaking {}
impl dp_staking::Stake for RingStaking {
	type AccountId = AccountId;
	type Item = Balance;

	fn stake(who: &Self::AccountId, item: Self::Item) -> sp_runtime::DispatchResult {
		<Balances as Currency<_>>::transfer(
			who,
			&dp_staking::account_id(),
			item,
			frame_support::traits::ExistenceRequirement::AllowDeath,
		)
	}

	fn unstake(who: &Self::AccountId, item: Self::Item) -> sp_runtime::DispatchResult {
		<Balances as Currency<_>>::transfer(
			&dp_staking::account_id(),
			who,
			item,
			frame_support::traits::ExistenceRequirement::AllowDeath,
		)
	}
}
pub enum KtonStaking {}
impl dp_staking::Stake for KtonStaking {
	type AccountId = AccountId;
	type Item = Balance;

	fn stake(who: &Self::AccountId, item: Self::Item) -> sp_runtime::DispatchResult {
		Assets::transfer(
			RuntimeOrigin::signed(*who),
			(AssetIds::CKton as AssetId).into(),
			dp_staking::account_id(),
			item,
		)
	}

	fn unstake(who: &Self::AccountId, item: Self::Item) -> sp_runtime::DispatchResult {
		Assets::transfer(
			RuntimeOrigin::signed(dp_staking::account_id()),
			(AssetIds::CKton as AssetId).into(),
			*who,
			item,
		)
	}
}

pub enum OnCrabSessionEnd {}
impl dp_staking::IssuingManager<Runtime> for OnCrabSessionEnd {
	fn calculate_reward(_inflation: Balance) -> Balance {
		20_000 * UNIT
	}

	fn reward(who: &AccountId, amount: Balance) -> sp_runtime::DispatchResult {
		<Balances as Currency<AccountId>>::transfer(
			&Treasury::account_id(),
			who,
			amount,
			frame_support::traits::ExistenceRequirement::KeepAlive,
		)
	}
}

pub enum ShouldEndSession {}
impl frame_support::traits::Get<bool> for ShouldEndSession {
	fn get() -> bool {
		// substrate
		use pallet_session::ShouldEndSession;

		<Runtime as pallet_session::Config>::ShouldEndSession::should_end_session(
			System::block_number(),
		)
	}
}

impl dp_staking::Config for Runtime {
	type Currency = Balances;
	type Deposit = Deposit;
	type IssuingManager = OnCrabSessionEnd;
	type Kton = KtonStaking;
	type KtonRewardDistributionContract = dp_staking::KtonRewardDistributionContract;
	type KtonStakerNotifier = dp_staking::KtonStakerNotifier<Self>;
	type MaxDeposits = <Self as dp_deposit::Config>::MaxDeposits;
	type MaxUnstakings = ConstU32<16>;
	type MigrationCurve = dp_staking::MigrationCurve<Self>;
	type MinStakingDuration = MinStakingDuration;
	type Ring = RingStaking;
	type RuntimeEvent = RuntimeEvent;
	type ShouldEndSession = ShouldEndSession;
	type WeightInfo = weights::dp_staking::WeightInfo<Self>;
}
#[cfg(not(feature = "runtime-benchmarks"))]
impl dp_staking::DepositConfig for Runtime {}
