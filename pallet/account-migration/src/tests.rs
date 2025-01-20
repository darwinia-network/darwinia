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
use crate::{mock::*, *};
// polkadot-sdk
use frame_support::{assert_noop, assert_ok, traits::OnIdle};
use frame_system::AccountInfo;
use pallet_balances::AccountData;
use sp_core::H256;
use sp_keyring::{ed25519::Keyring as Ek, sr25519::Keyring as Sk};
use sp_runtime::{
	traits::ValidateUnsigned,
	transaction_validity::{InvalidTransaction, TransactionValidityError},
};

#[test]
fn signable_message_should_work() {
	["Darwinia2", "Crab2"].iter().for_each(|s| {
		assert_eq!(
			signable_message(s.as_bytes(), &Default::default()),
			format!(
				"<Bytes>I authorize the migration to {}, an unused address on {}. Sign this message to authorize using the Substrate key associated with the account on {} that you wish to migrate.</Bytes>",
				"0x0000000000000000000000000000000000000000",
				s,
				&s[..s.len() - 1],
			).as_bytes()
		);
	});
}

#[test]
fn verify_curve_25519_signature_should_work() {
	Sk::iter().enumerate().for_each(|(i, from)| {
		let to = [i as _; 20];
		let message = signable_message(b"Darwinia2", &to.into());
		let signature = from.sign(&message);

		assert!(verify_curve_25519_signature(&from.public().0.into(), &message, &signature.0));
	});
	Ek::iter().enumerate().for_each(|(i, from)| {
		let to = [i as _; 20];
		let message = signable_message(b"Darwinia2", &to.into());
		let signature = from.sign(&message);

		assert!(verify_curve_25519_signature(&from.public().0.into(), &message, &signature.0));
	});
}

#[test]
fn multisig_of_should_work() {
	let (_, multisig) = multisig_of(
		// Xavier
		array_bytes::dehexify_array_then_into(
			"0xe66972adc51faaf978614e8eb4015e5536e236a05875cf9253dc421ed6c2ec4b",
		)
		.unwrap(),
		vec![
			// Alex
			array_bytes::dehexify_array_then_into(
				"0x26fe37ba5d35ac650ba37c5cc84525ed135e772063941ae221a1caca192fff49",
			)
			.unwrap(),
			// Denny
			array_bytes::dehexify_array_then_into(
				"0x0a66532a23c418cca12183fee5f6afece770a0bb8725f459d7d1b1b598f91c49",
			)
			.unwrap(),
		],
		2,
	);

	assert_eq!(
		multisig,
		// Sudo
		array_bytes::dehexify_array_then_into(
			"0xc778fc2665f3f6ee9623594e5d1fab9dbd557149542c5edacbcc543a82c9d780"
		)
		.unwrap()
	);
}

#[test]
fn migrate_multisig_should_work() {
	let a = Sk::Alice;
	let b = Sk::Bob;
	let c = Ek::Charlie;
	let d = Ek::Dave;
	let (_, multisig) =
		multisig_of(a.public().0.into(), vec![b.public().0.into(), c.public().0.into()], 2);
	let to = Default::default();
	let message = signable_message(b"Darwinia2", &to);

	new_test_ext().execute_with(|| {
		<Accounts<Runtime>>::insert(
			&multisig,
			AccountInfo::<Nonce, AccountData<Balance>> { consumers: 1, ..Default::default() },
		);

		assert!(<Multisigs<Runtime>>::get(&multisig).is_none());
		assert_eq!(<frame_system::Account<Runtime>>::get(to).consumers, 0);

		// Alice starts the migration.
		let signature = a.sign(&message);

		assert_ok!(AccountMigration::pre_dispatch(&Call::migrate_multisig {
			submitter: a.public().0.into(),
			others: vec![b.public().0.into(), c.public().0.into()],
			threshold: 2,
			to,
			signature: signature.0,
			new_multisig_params: None
		}));
		assert_ok!(AccountMigration::migrate_multisig(
			RuntimeOrigin::none(),
			a.public().0.into(),
			vec![b.public().0.into(), c.public().0.into()],
			2,
			to,
			signature.0,
			None
		));
		assert_noop!(
			AccountMigration::pre_dispatch(&Call::migrate_multisig {
				submitter: a.public().0.into(),
				others: vec![b.public().0.into(), c.public().0.into()],
				threshold: 2,
				to,
				signature: signature.0,
				new_multisig_params: None
			}),
			TransactionValidityError::Invalid(InvalidTransaction::Custom(E_DUPLICATED_SUBMISSION))
		);

		assert!(<Multisigs<Runtime>>::get(&multisig).is_some());
		assert_eq!(<frame_system::Account<Runtime>>::get(to).consumers, 0);

		// Dave tries to hack the multisig.
		let signature = d.sign(&message);

		assert_noop!(
			AccountMigration::pre_dispatch(&Call::complete_multisig_migration {
				multisig: multisig.clone(),
				submitter: d.public().0.into(),
				signature: signature.0
			}),
			TransactionValidityError::Invalid(InvalidTransaction::Custom(E_NOT_MULTISIG_MEMBER))
		);

		// Charlie completes the migration.
		let signature = c.sign(&message);

		assert_ok!(AccountMigration::pre_dispatch(&Call::complete_multisig_migration {
			multisig: multisig.clone(),
			submitter: c.public().0.into(),
			signature: signature.0
		}));
		assert_ok!(AccountMigration::complete_multisig_migration(
			RuntimeOrigin::none(),
			multisig.clone(),
			c.public().0.into(),
			signature.0
		));

		assert!(<Multisigs<Runtime>>::get(&multisig).is_none());
		assert_eq!(<frame_system::Account<Runtime>>::get(to).consumers, 1);
	});
}

#[test]
fn on_idle_should_work() {
	new_test_ext().execute_with(|| {
		(0..1_024).for_each(|i| {
			<Ledgers<Runtime>>::insert(
				AccountId32::from(H256::from_low_u64_le(i).0),
				OldLedger::default(),
			);
		});
		assert_eq!(<Ledgers<Runtime>>::iter().count(), 1_024);

		<AccountMigration as OnIdle<_>>::on_idle(0, Weight::zero().add_ref_time(5));
		assert_eq!(<Ledgers<Runtime>>::iter().count(), 1_019);

		<AccountMigration as OnIdle<_>>::on_idle(0, Weight::MAX);
		assert_eq!(<Ledgers<Runtime>>::iter().count(), 1_009);
	});
}
