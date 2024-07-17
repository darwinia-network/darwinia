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

#[macro_export]
macro_rules! impl_kton_staker_notifier_tracing {
	() => {
		pub struct KtonStakerNotifierTracing<T>(core::marker::PhantomData<T>);
		impl<T> darwinia_staking::KtonStakerNotification for KtonStakerNotifierTracing<T>
		where
			T: darwinia_staking::Config + darwinia_ethtx_forwarder::Config,
			T::RuntimeOrigin: Into<Result<darwinia_ethtx_forwarder::ForwardEthOrigin, T::RuntimeOrigin>>
				+ From<darwinia_ethtx_forwarder::ForwardEthOrigin>,
			<T as frame_system::Config>::AccountId: Into<H160>,
		{
			fn construct_notification(
				amount: Balance,
			) -> Option<darwinia_ethtx_forwarder::ForwardRequest> {
				darwinia_staking::KtonStakerNotifier::<T>::construct_notification(amount)
			}

			fn notify(
				sender: H160,
				notification: Option<darwinia_ethtx_forwarder::ForwardRequest>,
			) {
				if let Some(status) = frame_support::storage::unhashed::get::<
					xcm_primitives::EthereumXcmTracingStatus,
				>(xcm_primitives::ETHEREUM_XCM_TRACING_STORAGE_KEY)
				{
					match status {
						xcm_primitives::EthereumXcmTracingStatus::Block => {
							moonbeam_evm_tracer::tracer::EvmTracer::emit_new();
							moonbeam_evm_tracer::tracer::EvmTracer::new().trace(|| {
								darwinia_staking::KtonStakerNotifier::<T>::notify(
									sender,
									notification,
								)
							});
						},
						xcm_primitives::EthereumXcmTracingStatus::Transaction(
							traced_transaction_hash,
						) => {
							let transaction =
								darwinia_ethtx_forwarder::Pallet::<T>::validated_transaction(
									sender,
									notification
										.clone()
										.expect("notification must be a valid request"),
								)
								.expect("transaction should be valid");
							if transaction.hash() == traced_transaction_hash {
								moonbeam_evm_tracer::tracer::EvmTracer::new().trace(|| {
									darwinia_staking::KtonStakerNotifier::<T>::notify(
										sender,
										notification,
									)
								});
								frame_support::storage::unhashed::put::<
									xcm_primitives::EthereumXcmTracingStatus,
								>(
									xcm_primitives::ETHEREUM_XCM_TRACING_STORAGE_KEY,
									&xcm_primitives::EthereumXcmTracingStatus::TransactionExited,
								);
							}
						},
						xcm_primitives::EthereumXcmTracingStatus::TransactionExited => {},
					}
				} else {
					darwinia_staking::KtonStakerNotifier::<T>::notify(sender, notification)
				}
			}
		}
	};
}
