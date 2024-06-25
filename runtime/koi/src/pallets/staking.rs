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

pub enum RingStaking {}
impl darwinia_staking::Stake for RingStaking {
	type AccountId = AccountId;
	type Item = Balance;

	fn stake(who: &Self::AccountId, item: Self::Item) -> sp_runtime::DispatchResult {
		<Balances as frame_support::traits::Currency<_>>::transfer(
			who,
			&darwinia_staking::account_id(),
			item,
			frame_support::traits::ExistenceRequirement::AllowDeath,
		)
	}

	fn unstake(who: &Self::AccountId, item: Self::Item) -> sp_runtime::DispatchResult {
		<Balances as frame_support::traits::Currency<_>>::transfer(
			&darwinia_staking::account_id(),
			who,
			item,
			frame_support::traits::ExistenceRequirement::AllowDeath,
		)
	}
}
pub enum KtonStaking {}
impl darwinia_staking::Stake for KtonStaking {
	type AccountId = AccountId;
	type Item = Balance;

	fn stake(who: &Self::AccountId, item: Self::Item) -> sp_runtime::DispatchResult {
		Assets::transfer(
			RuntimeOrigin::signed(*who),
			(AssetIds::PKton as AssetId).into(),
			darwinia_staking::account_id(),
			item,
		)
	}

	fn unstake(who: &Self::AccountId, item: Self::Item) -> sp_runtime::DispatchResult {
		Assets::transfer(
			RuntimeOrigin::signed(darwinia_staking::account_id()),
			(AssetIds::PKton as AssetId).into(),
			*who,
			item,
		)
	}
}

pub enum OnKoiSessionEnd {}
impl darwinia_staking::IssuingManager<Runtime> for OnKoiSessionEnd {
	fn calculate_reward(_inflation: Balance) -> Balance {
		20_000 * UNIT
	}

	fn reward(who: &AccountId, amount: Balance) -> sp_runtime::DispatchResult {
		<Balances as frame_support::traits::Currency<AccountId>>::transfer(
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

impl darwinia_staking::Config for Runtime {
	type Currency = Balances;
	type Deposit = Deposit;
	type IssuingManager = OnKoiSessionEnd;
	type Kton = KtonStaking;
	type KtonRewardDistributionContract = darwinia_staking::KtonRewardDistributionContract;
	type KtonStakerNotifier = darwinia_staking::KtonStakerNotifier<Self>;
	type MaxDeposits = <Self as darwinia_deposit::Config>::MaxDeposits;
	type Ring = RingStaking;
	type RuntimeEvent = RuntimeEvent;
	type ShouldEndSession = ShouldEndSession;
	// type WeightInfo = weights::darwinia_staking::WeightInfo<Self>;
	type WeightInfo = ();
}
#[cfg(not(feature = "runtime-benchmarks"))]
impl darwinia_staking::DepositConfig for Runtime {}
