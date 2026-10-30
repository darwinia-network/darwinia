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

pub mod mock;

darwinia_common_runtime::impl_weight_tests! {}
darwinia_common_runtime::impl_fee_tests! {}
darwinia_common_runtime::impl_evm_tests! {}
// darwinia_common_runtime::impl_maintenance_tests! {}

#[test]
fn retired_account_migration_index_is_not_reused() {
	use frame_support::traits::PalletsInfoAccess;
	use mock::*;

	assert!(AllPalletsWithSystem::infos().iter().all(|pallet| pallet.index != 10));

	// The old call encoded index 10, call 0, a 32-byte source, a 20-byte
	// destination, and a 64-byte signature. Its index must remain invalid.
	let mut old_call = vec![10, 0];
	old_call.extend_from_slice(&[0; 32 + 20 + 64]);
	assert!(<RuntimeCall as codec::Decode>::decode(&mut old_call.as_slice()).is_err());
}

#[test]
fn non_canonical_asset_address_is_not_a_precompile() {
	use mock::*;
	use pallet_evm::Runner as _;
	use sp_core::{H160, U256};

	let caller = H160::repeat_byte(0xAA);
	let mut alias = H160::from_low_u64_be(KTON_ID);
	alias.0[11] = 1;

	ExtBuilder::default().build().execute_with(|| {
		let result = <Runtime as pallet_evm::Config>::Runner::call(
			caller,
			alias,
			vec![0x18, 0x16, 0x0D, 0xDD],
			U256::zero(),
			1_000_000,
			None,
			None,
			None,
			Vec::new(),
			false,
			false,
			None,
			None,
			<Runtime as pallet_evm::Config>::config(),
		)
		.expect("ordinary EVM call should execute");

		assert!(result.exit_reason.is_succeed());
		assert!(result.value.is_empty());
	});
}

#[test]
fn canonical_asset_precompile_still_executes() {
	use mock::*;
	use pallet_evm::Runner as _;
	use sp_core::{H160, U256};

	let caller = H160::repeat_byte(0xAA);
	let canonical = H160::from_low_u64_be(KTON_ID);

	ExtBuilder::default().build().execute_with(|| {
		let result = <Runtime as pallet_evm::Config>::Runner::call(
			caller,
			canonical,
			vec![0x18, 0x16, 0x0D, 0xDD],
			U256::zero(),
			1_000_000,
			None,
			None,
			None,
			Vec::new(),
			false,
			false,
			None,
			None,
			<Runtime as pallet_evm::Config>::config(),
		)
		.expect("canonical asset precompile call should execute");

		assert!(result.exit_reason.is_succeed());
		assert_eq!(result.value, vec![0; 32]);
	});
}
