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
use frame_benchmarking::v2;
use frame_system::RawOrigin;
use sp_std::prelude::*;

#[v2::benchmarks]
mod benchmarks {
	// darwinia
	use super::*;

	#[benchmark]
	fn on_initialize() {
		assert!(<NewMessageRootToSign<T>>::get().is_none());

		// The worst case:
		//
		// Trigger new message root.
		#[block]
		{
			<Pallet<T>>::on_initialize(Default::default());
		}

		assert!(<NewMessageRootToSign<T>>::get().is_some());
	}

	#[benchmark]
	fn add_authority() {
		// The worst case:
		//
		// Add the authority successfully.
		#[extrinsic_call]
		_(RawOrigin::Root, frame_benchmarking::account("", 0, 0));
	}

	#[benchmark]
	fn remove_authority() {
		let a = frame_benchmarking::account("", 0, 0);

		<Pallet<T>>::add_authority(RawOrigin::Root.into(), a).unwrap();
		<Pallet<T>>::presume_authority_change_succeed();
		<Pallet<T>>::add_authority(RawOrigin::Root.into(), frame_benchmarking::account("", 1, 1))
			.unwrap();
		<Pallet<T>>::presume_authority_change_succeed();

		// The worst case:
		//
		// Remove the authority successfully.
		#[extrinsic_call]
		_(RawOrigin::Root, a);
	}

	#[benchmark]
	fn swap_authority() {
		let x = T::MaxAuthorities::get();

		(0..x).for_each(|i| {
			<Pallet<T>>::add_authority(
				RawOrigin::Root.into(),
				frame_benchmarking::account("", i, i),
			)
			.unwrap();
			<Pallet<T>>::presume_authority_change_succeed();
		});

		let old = frame_benchmarking::account("", x - 1, x - 1);
		let new = frame_benchmarking::account("", x, x);

		// The worst case:
		//
		// Swap the last authority successfully.
		#[extrinsic_call]
		_(RawOrigin::Root, old, new);
	}

	#[benchmark]
	fn submit_authorities_change_signature() {
		// https://github.com/paritytech/libsecp256k1/issues/134
		// let (sk, pk) = gen_pair(1);
		let pk = frame_benchmarking::account("", 777, 777);
		let a = frame_benchmarking::account("", 0, 0);
		let data = AuthoritiesChangeSigned {
			operation: Operation::AddMember { new: a },
			threshold: Default::default(),
			message: Default::default(),
			signatures: Default::default(),
		};
		// let sig = sign(&sk, &data.2 .0);
		let sig = Default::default();

		<Pallet<T>>::add_authority(RawOrigin::Root.into(), pk).unwrap();
		<Pallet<T>>::presume_authority_change_succeed();
		<Pallet<T>>::add_authority(RawOrigin::Root.into(), a).unwrap();
		<AuthoritiesChangeToSign<T>>::put(data);

		// The worst case:
		//
		// Submit the signature and pass the threshold checking successfully.
		#[extrinsic_call]
		_(RawOrigin::Signed(pk), sig);
	}

	#[benchmark]
	fn submit_new_message_root_signature() {
		// https://github.com/paritytech/libsecp256k1/issues/134
		// let (sk, pk) = gen_pair(1);
		let pk = frame_benchmarking::account("", 777, 777);
		let data = MessageRootSigned {
			commitment: Commitment {
				block_number: Default::default(),
				message_root: Default::default(),
				nonce: Default::default(),
			},
			message: Default::default(),
			signatures: Default::default(),
			authorized: Default::default(),
		};
		// let sig = sign(&sk, &data.1 .0);
		let sig = Default::default();

		<Pallet<T>>::add_authority(RawOrigin::Root.into(), pk).unwrap();
		<Pallet<T>>::presume_authority_change_succeed();
		<MessageRootToSign<T>>::put(data);

		// The worst case:
		//
		// Submit the signature and pass the threshold checking successfully.
		#[extrinsic_call]
		_(RawOrigin::Signed(pk), sig);
	}

	frame_benchmarking::impl_benchmark_test_suite!(
		Pallet,
		crate::mock::ExtBuilder::default().build(),
		crate::mock::Runtime
	);
}
