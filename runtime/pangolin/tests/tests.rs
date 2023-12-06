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
darwinia_common_runtime::impl_evm_tests! {}
darwinia_common_runtime::impl_account_migration_tests! {}
darwinia_common_runtime::impl_messages_bridge_tests! {}

mod maintain_test {
	// darwinia
	use super::mock::*;
	use frame_support::{assert_err, assert_ok};
	use pallet_tx_pause::RuntimeCallNameOf;
	use sp_core::{H160, U256};
	use sp_runtime::{traits::Dispatchable, DispatchError, ModuleError};

	pub fn full_name(pallet_name: &[u8], call_name: &[u8]) -> RuntimeCallNameOf<Runtime> {
		<RuntimeCallNameOf<Runtime>>::from((
			pallet_name.to_vec().try_into().unwrap(),
			call_name.to_vec().try_into().unwrap(),
		))
	}

	#[test]
	fn tx_pause_origins_work_correctly() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(TxPause::pause(RuntimeOrigin::root(), full_name(b"Balances", b"transfer")));

			assert_err!(
				TxPause::pause(
					RuntimeOrigin::signed(H160::default().into()),
					full_name(b"Balances", b"transfer")
				),
				DispatchError::BadOrigin
			);
		})
	}

	#[test]
	fn tx_pause_pause_and_unpause_work_correctly() {
		let from = H160::from_low_u64_be(555).into();
		let to = H160::from_low_u64_be(333).into();
		ExtBuilder::default().with_balances(vec![(from, 100), (to, 50)]).build().execute_with(
			|| {
				assert_ok!(TxPause::pause(RuntimeOrigin::root(), full_name(b"System", b"remark")));
				assert_err!(
					RuntimeCall::System(frame_system::Call::remark {
						remark: b"hello world".to_vec()
					})
					.dispatch(RuntimeOrigin::signed(from)),
					frame_system::Error::<Runtime>::CallFiltered
				);

				assert_ok!(TxPause::unpause(
					RuntimeOrigin::root(),
					full_name(b"System", b"remark")
				));
				assert_ok!(RuntimeCall::System(frame_system::Call::remark {
					remark: b"hello world".to_vec(),
				})
				.dispatch(RuntimeOrigin::signed(from)));
			},
		)
	}

	#[test]
	fn tx_pause_pause_calls_except_on_whitelist() {
		let from = H160::from_low_u64_be(555).into();
		ExtBuilder::default().with_balances(vec![(from, 100)]).build().execute_with(|| {
			assert_ok!(RuntimeCall::System(frame_system::Call::remark_with_event {
				remark: b"hello world".to_vec(),
			})
			.dispatch(RuntimeOrigin::signed(from)));

			assert_err!(
				TxPause::pause(RuntimeOrigin::root(), full_name(b"System", b"remark_with_event")),
				pallet_tx_pause::Error::<Runtime>::Unpausable
			);

			assert_ok!(RuntimeCall::System(frame_system::Call::remark_with_event {
				remark: b"hello world".to_vec(),
			})
			.dispatch(RuntimeOrigin::signed(from)));
		})
	}
}
