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

mod mock;
use mock::*;

// darwinia
use darwinia_ecdsa_authority::{primitives::*, *};
// substrate
use frame_support::{
	assert_noop, assert_ok,
	dispatch::{Pays, PostDispatchInfo},
	traits::Get,
	BoundedVec,
};
use sp_runtime::DispatchError;

#[test]
fn calculate_threshold() {
	ExtBuilder::default().build().execute_with(|| {
		for (expected, i) in [(1, 1), (2, 2), (2, 3), (3, 4), (3, 5), (4, 6)] {
			assert_eq!(EcdsaAuthority::calculate_threshold(i), expected);
		}
	});
}

#[test]
fn add_authority() {
	let a_0 = account_id_of(0);

	ExtBuilder::default().build().execute_with(|| {
		assert!(EcdsaAuthority::authorities().is_empty());
		assert!(EcdsaAuthority::next_authorities().is_empty());
		assert_eq!(EcdsaAuthority::nonce(), 0);
		assert_ok!(EcdsaAuthority::add_authority(RuntimeOrigin::root(), a_0));
		assert!(EcdsaAuthority::authorities().is_empty());
		assert_eq!(EcdsaAuthority::next_authorities(), vec![a_0]);
		assert_eq!(EcdsaAuthority::nonce(), 0);
		let message = array_bytes::hex_n_into_unchecked(
			"0x5c883184c9c53c59857253454df1b4813e8b3fb28648beb85555d58d1e801e14",
		);
		assert_eq!(
			EcdsaAuthority::authorities_change_to_sign(),
			Some((Operation::AddMember { new: a_0 }, Some(1), message, Default::default()))
		);
		assert_eq!(
			ecdsa_authority_events(),
			vec![Event::CollectingAuthoritiesChangeSignatures { message }]
		);

		// Case 1.
		assert_noop!(
			EcdsaAuthority::add_authority(RuntimeOrigin::root(), a_0),
			<Error<Runtime>>::OnAuthoritiesChange
		);
		presume_authority_change_succeed();
		assert_eq!(EcdsaAuthority::authorities(), vec![a_0]);
		assert_eq!(EcdsaAuthority::nonce(), 1);

		// Case 2.
		assert_noop!(
			EcdsaAuthority::add_authority(RuntimeOrigin::signed(Default::default()), a_0),
			DispatchError::BadOrigin
		);

		// Case 3.
		assert_noop!(
			EcdsaAuthority::add_authority(RuntimeOrigin::root(), a_0),
			<Error<Runtime>>::AuthorityExisted
		);

		// Case 4.
		(1..<Runtime as Config>::MaxAuthorities::get()).for_each(|i| {
			assert_ok!(EcdsaAuthority::add_authority(RuntimeOrigin::root(), account_id_of(i as _)));
			presume_authority_change_succeed();
			assert_eq!(EcdsaAuthority::nonce(), 1 + i);
		});
		assert_noop!(
			EcdsaAuthority::add_authority(
				RuntimeOrigin::root(),
				account_id_of(<<Runtime as Config>::MaxAuthorities as Get<u32>>::get() as _)
			),
			<Error<Runtime>>::TooManyAuthorities
		);

		// Check order.
		assert_eq!(
			EcdsaAuthority::authorities(),
			(0..<Runtime as Config>::MaxAuthorities::get())
				.rev()
				.map(|i| account_id_of(i as _))
				.collect::<Vec<_>>()
		);
	});
}

#[test]
fn remove_authority() {
	let a_1 = account_id_of(1);
	let a_2 = account_id_of(2);

	ExtBuilder::default().authorities(vec![a_1, a_2]).build().execute_with(|| {
		assert_eq!(EcdsaAuthority::authorities(), vec![a_1, a_2]);
		assert_eq!(EcdsaAuthority::next_authorities(), vec![a_1, a_2]);
		assert_eq!(EcdsaAuthority::nonce(), 0);
		assert_ok!(EcdsaAuthority::remove_authority(RuntimeOrigin::root(), a_1));
		assert_eq!(EcdsaAuthority::authorities(), vec![a_1, a_2]);
		assert_eq!(EcdsaAuthority::next_authorities(), vec![a_2]);
		assert_eq!(EcdsaAuthority::nonce(), 0);
		let message = array_bytes::hex_n_into_unchecked(
			"0x76139aa9d1c7b35fc744b10444898ee5703e3f77406b926f903006436b7930c7",
		);
		assert_eq!(
			EcdsaAuthority::authorities_change_to_sign(),
			Some((
				Operation::RemoveMember { pre: AUTHORITY_SENTINEL.into(), old: a_1 },
				Some(1),
				message,
				Default::default()
			))
		);
		assert_eq!(
			ecdsa_authority_events(),
			vec![Event::CollectingAuthoritiesChangeSignatures { message }]
		);

		// Case 1.
		assert_noop!(
			EcdsaAuthority::add_authority(RuntimeOrigin::root(), a_1),
			<Error<Runtime>>::OnAuthoritiesChange
		);
		presume_authority_change_succeed();
		assert_eq!(EcdsaAuthority::authorities(), vec![a_2]);
		assert_eq!(EcdsaAuthority::nonce(), 1);

		// Case 2.
		assert_noop!(
			EcdsaAuthority::remove_authority(RuntimeOrigin::signed(Default::default()), a_2),
			DispatchError::BadOrigin
		);

		// Case 3.
		assert_noop!(
			EcdsaAuthority::remove_authority(RuntimeOrigin::root(), a_1),
			<Error<Runtime>>::NotAuthority
		);

		// Case 4.
		assert_noop!(
			EcdsaAuthority::remove_authority(RuntimeOrigin::root(), a_2),
			<Error<Runtime>>::AtLeastOneAuthority
		);
	});
}

#[test]
fn swap_authority() {
	let a_1 = account_id_of(1);
	let a_2 = account_id_of(2);

	ExtBuilder::default().authorities(vec![a_1]).build().execute_with(|| {
		assert_eq!(EcdsaAuthority::authorities(), vec![a_1]);
		assert_eq!(EcdsaAuthority::next_authorities(), vec![a_1]);
		assert_eq!(EcdsaAuthority::nonce(), 0);
		assert_ok!(EcdsaAuthority::swap_authority(RuntimeOrigin::root(), a_1, a_2));
		assert_eq!(EcdsaAuthority::authorities(), vec![a_1]);
		assert_eq!(EcdsaAuthority::next_authorities(), vec![a_2]);
		assert_eq!(EcdsaAuthority::nonce(), 0);
		let message = array_bytes::hex_n_into_unchecked(
			"0x30effc17a3fcf9b3079168c2c2be54b6d9fbdfd7077c9d844ec241dd70dd0507",
		);
		assert_eq!(
			EcdsaAuthority::authorities_change_to_sign(),
			Some((
				Operation::SwapMembers { pre: AUTHORITY_SENTINEL.into(), old: a_1, new: a_2 },
				None,
				message,
				Default::default()
			))
		);
		assert_eq!(
			ecdsa_authority_events(),
			vec![Event::CollectingAuthoritiesChangeSignatures { message }]
		);

		// Case 1.
		assert_noop!(
			EcdsaAuthority::swap_authority(RuntimeOrigin::root(), a_2, a_1),
			<Error<Runtime>>::OnAuthoritiesChange
		);
		presume_authority_change_succeed();
		assert_eq!(EcdsaAuthority::authorities(), vec![a_2]);
		assert_eq!(EcdsaAuthority::nonce(), 1);

		// Case 2.
		assert_noop!(
			EcdsaAuthority::swap_authority(RuntimeOrigin::signed(Default::default()), a_2, a_1),
			DispatchError::BadOrigin
		);

		// Case 3.
		assert_noop!(
			EcdsaAuthority::swap_authority(RuntimeOrigin::root(), a_1, a_2),
			<Error<Runtime>>::NotAuthority
		);
	});
}

#[test]
fn sync_interval_and_max_pending_period() {
	ExtBuilder::default().build().execute_with(|| {
		// Check new message root while reaching the sync interval checkpoint.
		(2..<Runtime as Config>::SyncInterval::get()).for_each(|i| {
			run_to_block(i as _);
			assert!(EcdsaAuthority::new_message_root_to_sign().is_none());
		});
		run_to_block(<Runtime as Config>::SyncInterval::get());
		let message = array_bytes::hex_n_into_unchecked(
			"0x742776a31e49b3f5a2a15a6781eb99f96e8116bfc67aae652a08b9b1235146d2",
		);
		assert_eq!(
			EcdsaAuthority::new_message_root_to_sign(),
			Some((
				Commitment {
					block_number: System::block_number() as _,
					message_root: Default::default(),
					nonce: 0
				},
				message,
				Default::default()
			))
		);
		assert_eq!(
			ecdsa_authority_events(),
			vec![Event::CollectingNewMessageRootSignatures { message }]
		);

		// Use a new message root while exceeding the max pending period.
		new_message_root(1);
		let offset = System::block_number() + 1;
		(offset..offset + <<Runtime as Config>::MaxPendingPeriod as Get<u64>>::get()).for_each(
			|i| {
				run_to_block(i);
				assert_eq!(
					EcdsaAuthority::new_message_root_to_sign(),
					Some((
						Commitment { block_number: 3, message_root: Default::default(), nonce: 0 },
						message,
						Default::default()
					))
				);
			},
		);
		run_to_block(offset + <<Runtime as Config>::MaxPendingPeriod as Get<u64>>::get());
		let message = array_bytes::hex_n_into_unchecked(
			"0xafd9fc3dc135079be23746b4beb27255e5d3b4c5f3d05db766af751c0ed97920",
		);
		assert_eq!(
			EcdsaAuthority::new_message_root_to_sign(),
			Some((
				Commitment { block_number: 9, message_root: message_root_of(1), nonce: 0 },
				message,
				Default::default()
			))
		);

		// Not allow to update the message root while authorities changing.
		assert_ok!(EcdsaAuthority::add_authority(RuntimeOrigin::root(), Default::default()));
		new_message_root(2);
		let offset = System::block_number() + 1;
		(offset..=offset + <<Runtime as Config>::MaxPendingPeriod as Get<u64>>::get()).for_each(
			|i| {
				run_to_block(i);
				assert_eq!(
					EcdsaAuthority::new_message_root_to_sign(),
					Some((
						Commitment { block_number: 9, message_root: message_root_of(1), nonce: 0 },
						message,
						Default::default()
					))
				);
			},
		);
	});
}

#[test]
fn submit_authorities_change_signature() {
	let (k_1, a_1) = gen_pair(1);
	let (k_2, a_2) = gen_pair(2);
	let (_, a_3) = gen_pair(3);

	ExtBuilder::default().authorities(vec![a_1, a_2]).build().execute_with(|| {
		// Case 1.
		assert_noop!(
			EcdsaAuthority::submit_authorities_change_signature(
				RuntimeOrigin::signed(a_1),
				Default::default(),
			),
			<Error<Runtime>>::NoAuthoritiesChange
		);

		assert_ok!(EcdsaAuthority::add_authority(RuntimeOrigin::root(), a_3));
		let operation = Operation::AddMember { new: a_3 };
		let message = array_bytes::hex_n_into_unchecked(
			"0x3ad89c7824d6e83c180482c888a0af99baa95ce17a39285d6f943df5d95e7759",
		);
		assert_eq!(
			EcdsaAuthority::authorities_change_to_sign(),
			Some((operation.clone(), Some(2), message, Default::default()))
		);
		assert_eq!(
			ecdsa_authority_events(),
			vec![Event::CollectingAuthoritiesChangeSignatures { message }]
		);

		// Case 2.
		assert_noop!(
			EcdsaAuthority::submit_authorities_change_signature(
				RuntimeOrigin::signed(a_1),
				Default::default(),
			),
			<Error<Runtime>>::BadSignature
		);

		let nonce = EcdsaAuthority::nonce();
		let s_1 = sign(&k_1, &message.0);
		assert_eq!(EcdsaAuthority::nonce(), nonce);
		assert_ok!(EcdsaAuthority::submit_authorities_change_signature(
			RuntimeOrigin::signed(a_1),
			s_1.clone(),
		));
		assert_eq!(
			EcdsaAuthority::authorities_change_to_sign(),
			Some((
				operation.clone(),
				Some(2),
				message,
				BoundedVec::try_from(vec![(a_1, s_1.clone())]).unwrap()
			))
		);

		let s_2 = sign(&k_2, &message.0);
		assert_ok!(EcdsaAuthority::submit_authorities_change_signature(
			RuntimeOrigin::signed(a_2),
			s_2.clone(),
		));
		assert_eq!(EcdsaAuthority::nonce(), nonce + 1);
		assert!(EcdsaAuthority::authorities_change_to_sign().is_none());
		assert_eq!(
			ecdsa_authority_events(),
			vec![
				Event::CollectedEnoughAuthoritiesChangeSignatures {
					operation,
					new_threshold: Some(2),
					message,
					signatures: vec![(a_1, s_1), (a_2, s_2)]
				},
				Event::CollectingNewMessageRootSignatures {
					message: array_bytes::hex_n_into_unchecked(
						"0xe7bded73843f446f46b42ee0e0cc435f4f66fbcedf36c635c437a4d63bb44696"
					)
				}
			]
		);
	});
}

#[test]
fn submit_new_message_root_signature() {
	let (k_1, a_1) = gen_pair(1);
	let (k_2, a_2) = gen_pair(2);
	let (k_3, a_3) = gen_pair(3);

	ExtBuilder::default().authorities(vec![a_1, a_2]).build().execute_with(|| {
		// Case 1.
		assert_noop!(
			EcdsaAuthority::submit_new_message_root_signature(
				RuntimeOrigin::signed(a_1),
				Default::default(),
			),
			<Error<Runtime>>::NoNewMessageRoot
		);

		run_to_block(<<Runtime as Config>::SyncInterval as Get<u64>>::get());
		let message = array_bytes::hex_n_into_unchecked(
			"0x742776a31e49b3f5a2a15a6781eb99f96e8116bfc67aae652a08b9b1235146d2",
		);
		assert_eq!(
			EcdsaAuthority::new_message_root_to_sign(),
			Some((
				Commitment {
					block_number: System::block_number() as _,
					message_root: Default::default(),
					nonce: 0
				},
				message,
				Default::default()
			))
		);
		assert_eq!(
			ecdsa_authority_events(),
			vec![Event::CollectingNewMessageRootSignatures { message }]
		);

		// Case 2.
		assert_noop!(
			EcdsaAuthority::submit_new_message_root_signature(
				RuntimeOrigin::signed(a_1),
				Default::default(),
			),
			<Error<Runtime>>::BadSignature
		);

		// Case 3.
		let s_3 = sign(&k_3, &message.0);
		assert_noop!(
			EcdsaAuthority::submit_new_message_root_signature(RuntimeOrigin::signed(a_3), s_3,),
			<Error<Runtime>>::NotAuthority
		);

		let nonce = EcdsaAuthority::nonce();
		let s_1 = sign(&k_1, &message.0);
		assert_eq!(EcdsaAuthority::nonce(), nonce);
		assert_ok!(EcdsaAuthority::submit_new_message_root_signature(
			RuntimeOrigin::signed(a_1),
			s_1.clone(),
		));
		assert_eq!(
			EcdsaAuthority::new_message_root_to_sign(),
			Some((
				Commitment {
					block_number: System::block_number() as _,
					message_root: Default::default(),
					nonce: 0
				},
				message,
				BoundedVec::try_from(vec![(a_1, s_1.clone())]).unwrap()
			))
		);

		let s_2 = sign(&k_2, &message.0);
		assert_ok!(EcdsaAuthority::submit_new_message_root_signature(
			RuntimeOrigin::signed(a_2),
			s_2.clone(),
		));
		assert_eq!(EcdsaAuthority::nonce(), nonce);
		assert!(EcdsaAuthority::new_message_root_to_sign().is_none());
		assert_eq!(
			ecdsa_authority_events(),
			vec![Event::CollectedEnoughNewMessageRootSignatures {
				commitment: Commitment {
					block_number: System::block_number() as _,
					message_root: Default::default(),
					nonce: EcdsaAuthority::nonce()
				},
				message,
				signatures: vec![(a_1, s_1), (a_2, s_2)]
			}]
		);
		assert!(EcdsaAuthority::new_message_root_to_sign().is_none());
		assert!(EcdsaAuthority::previous_message_root().is_none());
	});
}

#[test]
fn tx_fee() {
	let (k_1, a_1) = gen_pair(1);
	let (_, a_2) = gen_pair(2);

	ExtBuilder::default().authorities(vec![a_1, a_2]).build().execute_with(|| {
		(2..<Runtime as Config>::SyncInterval::get()).for_each(|n| run_to_block(n as _));
		run_to_block(<<Runtime as Config>::SyncInterval as Get<u64>>::get());
		let message = array_bytes::hex_n_into_unchecked(
			"0x742776a31e49b3f5a2a15a6781eb99f96e8116bfc67aae652a08b9b1235146d2",
		);

		// Free for first-correct signature.
		assert_eq!(
			EcdsaAuthority::submit_new_message_root_signature(
				RuntimeOrigin::signed(a_1),
				sign(&k_1, &message),
			),
			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::No })
		);

		// Forbidden for submitting multiple times once the previous one succeeds.
		assert_noop!(
			EcdsaAuthority::submit_new_message_root_signature(
				RuntimeOrigin::signed(a_1),
				Default::default(),
			),
			<Error<Runtime>>::AlreadySubmitted
		);

		assert_ok!(EcdsaAuthority::remove_authority(RuntimeOrigin::root(), a_1));
		let message = array_bytes::hex_n_into_unchecked(
			"0x24956af4b0842e1caec63782602c5a94089ba7c8ab8bd12d4243bb1a893b8af0",
		);

		// Free for first-correct signature.
		assert_eq!(
			EcdsaAuthority::submit_authorities_change_signature(
				RuntimeOrigin::signed(a_1),
				sign(&k_1, &message),
			),
			Ok(PostDispatchInfo { actual_weight: None, pays_fee: Pays::No })
		);

		// Forbidden for submitting multiple times once the previous one succeeds.
		assert_noop!(
			EcdsaAuthority::submit_authorities_change_signature(
				RuntimeOrigin::signed(a_1),
				Default::default(),
			),
			<Error<Runtime>>::AlreadySubmitted
		);
	});
}
