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
// substrate
use frame_support::traits::Currency;

fast_runtime_or_not!(DURATION, BlockNumber, 5 * MINUTES, 14 * DAYS);

type MinStakingDuration = ConstU32<{ DURATION }>;

pub enum RingStaking {}
impl darwinia_staking::Stake for RingStaking {
	type AccountId = AccountId;
	type Item = Balance;

	fn stake(who: &Self::AccountId, item: Self::Item) -> sp_runtime::DispatchResult {
		<Balances as Currency<_>>::transfer(
			who,
			&darwinia_staking::account_id(),
			item,
			frame_support::traits::ExistenceRequirement::AllowDeath,
		)
	}

	fn unstake(who: &Self::AccountId, item: Self::Item) -> sp_runtime::DispatchResult {
		<Balances as Currency<_>>::transfer(
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
			(AssetIds::Kton as AssetId).into(),
			darwinia_staking::account_id(),
			item,
		)
	}

	fn unstake(who: &Self::AccountId, item: Self::Item) -> sp_runtime::DispatchResult {
		Assets::transfer(
			RuntimeOrigin::signed(darwinia_staking::account_id()),
			(AssetIds::Kton as AssetId).into(),
			*who,
			item,
		)
	}
}

pub enum OnDarwiniaSessionEnd {}
impl darwinia_staking::InflationManager<Runtime> for OnDarwiniaSessionEnd {
	fn inflate() -> Balance {
		let now = Timestamp::now() as Moment;
		let session_duration = now - <darwinia_staking::SessionStartTime<Runtime>>::get();
		let elapsed_time = <darwinia_staking::ElapsedTime<Runtime>>::mutate(|t| {
			*t = t.saturating_add(session_duration);

			*t
		});

		<darwinia_staking::SessionStartTime<Runtime>>::put(now);

		let unminted = dc_inflation::TOTAL_SUPPLY.saturating_sub(Balances::total_issuance());

		dc_inflation::in_period(unminted, session_duration, elapsed_time).unwrap_or_default()
	}

	fn calculate_reward(inflation: Balance) -> Balance {
		sp_runtime::Perbill::from_percent(40) * inflation
	}

	fn reward(who: &AccountId, amount: Balance) -> sp_runtime::DispatchResult {
		let _ = Balances::deposit_into_existing(who, amount)?;

		Ok(())
	}

	fn clear(remaining: Balance) {
		let _ = Balances::deposit_into_existing(&Treasury::account_id(), remaining);
	}
}

impl darwinia_staking::Config for Runtime {
	type Currency = Balances;
	type Deposit = Deposit;
	type InflationManager = OnDarwiniaSessionEnd;
	type Kton = KtonStaking;
	type MaxDeposits = <Self as darwinia_deposit::Config>::MaxDeposits;
	type MaxUnstakings = ConstU32<16>;
	type MinStakingDuration = MinStakingDuration;
	type Ring = RingStaking;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::darwinia_staking::WeightInfo<Self>;
}
#[cfg(not(feature = "runtime-benchmarks"))]
impl darwinia_staking::DepositConfig for Runtime {}
