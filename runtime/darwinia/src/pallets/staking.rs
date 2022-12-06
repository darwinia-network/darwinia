// This file is part of Darwinia.
//
// Copyright (C) 2018-2022 Darwinia Network
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
			frame_support::traits::ExistenceRequirement::KeepAlive,
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
		Assets::transfer(RuntimeOrigin::signed(*who), 0, darwinia_staking::account_id(), item)
	}

	fn unstake(who: &Self::AccountId, item: Self::Item) -> sp_runtime::DispatchResult {
		Assets::transfer(RuntimeOrigin::signed(darwinia_staking::account_id()), 0, *who, item)
	}
}

frame_support::parameter_types! {
	pub const PayoutFraction: sp_runtime::Perbill = sp_runtime::Perbill::from_percent(20);
}

impl darwinia_staking::Config for Runtime {
	type Deposit = Deposit;
	type Kton = KtonStaking;
	type MaxDeposits = ConstU32<16>;
	type MaxUnstakings = ConstU32<16>;
	type MinStakingDuration = ConstU32<{ 14 * DAYS }>;
	type PayoutFraction = PayoutFraction;
	type RewardRemainder = Treasury;
	type Ring = RingStaking;
	type RingCurrency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type UnixTime = Timestamp;
}
