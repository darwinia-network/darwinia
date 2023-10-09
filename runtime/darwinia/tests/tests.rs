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

pub mod mock;

darwinia_common_runtime::impl_weight_tests! {}
darwinia_common_runtime::impl_fee_tests! {}
darwinia_common_runtime::impl_ethereum_tests! {}
darwinia_common_runtime::impl_account_migration_tests! {}
darwinia_common_runtime::impl_messages_bridge_tests! {}
darwinia_common_runtime::impl_governance_tests! {}
darwinia_common_runtime::impl_balances_tests! {}
darwinia_common_runtime::impl_assets_tests! {}
darwinia_common_runtime::impl_message_transact_tests! {}

mod specific_setting {
	// darwinia
	use crate::mock::*;
	// substrate
	use frame_support::traits::Get;

	#[test]
	fn precompile_address() {
		assert_eq!(
			DarwiniaPrecompiles::<Runtime>::used_addresses()
				.iter()
				.map(|a| a.to_low_u64_be())
				.collect::<Vec<u64>>(),
			vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 1024, 1025, 1026, 1536, 1537, 2048]
		);
	}

	#[test]
	fn democracy_mod() {
		assert_eq!(
			<<Runtime as pallet_democracy::Config>::CooloffPeriod as Get<u32>>::get(),
			7 * DAYS
		);
		assert_eq!(
			<<Runtime as pallet_democracy::Config>::EnactmentPeriod as Get<u32>>::get(),
			28 * DAYS
		);
		assert_eq!(
			<<Runtime as pallet_democracy::Config>::FastTrackVotingPeriod as Get<u32>>::get(),
			3 * HOURS
		);
		assert_eq!(
			<<Runtime as pallet_democracy::Config>::LaunchPeriod as Get<u32>>::get(),
			28 * DAYS
		);
		assert_eq!(
			<<Runtime as pallet_democracy::Config>::VoteLockingPeriod as Get<u32>>::get(),
			28 * DAYS
		);
		assert_eq!(
			<<Runtime as pallet_democracy::Config>::VotingPeriod as Get<u32>>::get(),
			28 * DAYS
		);
	}

	#[test]
	fn collective_mod() {
		assert_eq!(
			<<Runtime as pallet_collective::Config<CouncilCollective>>::MotionDuration as Get<
				u32,
			>>::get(),
			7 * DAYS
		);
		assert_eq!(
			<<Runtime as pallet_collective::Config<TechnicalCollective>>::MotionDuration as Get<
				u32,
			>>::get(),
			7 * DAYS
		);
	}

	#[test]
	fn treasury_mod() {
		assert_eq!(
			<<Runtime as pallet_treasury::Config>::SpendPeriod as Get<u32>>::get(),
			24 * DAYS
		);
	}
}
