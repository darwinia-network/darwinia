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
// polkadot-sdk
use frame_support::traits::Currency;

fast_runtime_or_not!(DURATION, BlockNumber, 5 * MINUTES, 14 * DAYS);
#[cfg(feature = "evm-tracing")]
darwinia_common_runtime::impl_kton_staker_notifier_tracing!();

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
impl darwinia_staking::IssuingManager<Runtime> for OnDarwiniaSessionEnd {
	fn inflate() -> Balance {
		let now = Timestamp::now() as Moment;
		let session_duration = now - <darwinia_staking::SessionStartTime<Runtime>>::get();
		let elapsed_time = <darwinia_staking::ElapsedTime<Runtime>>::mutate(|t| {
			*t = t.saturating_add(session_duration);

			*t
		});

		<darwinia_staking::SessionStartTime<Runtime>>::put(now);

		dc_inflation::issuing_in_period(session_duration, elapsed_time).unwrap_or_default()
	}

	fn calculate_reward(issued: Balance) -> Balance {
		sp_runtime::Perbill::from_percent(40) * issued
	}

	fn reward(who: &AccountId, amount: Balance) -> sp_runtime::DispatchResult {
		let _ = Balances::deposit_creating(who, amount);

		Ok(())
	}
}

pub enum ShouldEndSession {}
impl frame_support::traits::Get<bool> for ShouldEndSession {
	fn get() -> bool {
		// polkadot-sdk
		use pallet_session::ShouldEndSession;

		<Runtime as pallet_session::Config>::ShouldEndSession::should_end_session(
			System::block_number(),
		)
	}
}

impl darwinia_staking::Config for Runtime {
	type Currency = Balances;
	type Deposit = Deposit;
	type IssuingManager = OnDarwiniaSessionEnd;
	type Kton = KtonStaking;
	type KtonRewardDistributionContract = darwinia_staking::KtonRewardDistributionContract;
	#[cfg(not(feature = "evm-tracing"))]
	type KtonStakerNotifier = darwinia_staking::KtonStakerNotifier<Self>;
	#[cfg(feature = "evm-tracing")]
	type KtonStakerNotifier = KtonStakerNotifierTracing<Self>;
	type MaxDeposits = <Self as darwinia_deposit::Config>::MaxDeposits;
	type Ring = RingStaking;
	type RuntimeEvent = RuntimeEvent;
	type ShouldEndSession = ShouldEndSession;
	type WeightInfo = weights::darwinia_staking::WeightInfo<Self>;
}
#[cfg(not(feature = "runtime-benchmarks"))]
impl darwinia_staking::DepositConfig for Runtime {}
