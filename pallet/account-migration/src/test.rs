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
use crate::{mock::*, ClaimMessage, Error};
// substrate
use frame_support::{assert_err, assert_ok, unsigned::ValidateUnsigned};
use sp_core::{blake2_256, Pair, H160};
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidityError};

#[test]
fn claim_to_new_account() {
	let (pair, charlie) = SubAccounts::Charlie.to_pair();
	let alice: H160 = EthAccounts::Alice.into();

	ExtBuilder::default()
		.with_migrated_accounts(vec![(charlie.clone(), 1000)])
		.build()
		.execute_with(|| {
			let message = ClaimMessage::new(42, &charlie, &alice);
			let sig = pair.sign(&blake2_256(&message.raw_bytes())[..]);

			assert_eq!(AccountMigration::balance_of(charlie.clone()), Some(1000));
			assert_eq!(Balances::free_balance(alice), 0);
			assert_ok!(AccountMigration::claim_to(
				RuntimeOrigin::none(),
				42,
				charlie.clone(),
				alice,
				sig
			));
			assert!(AccountMigration::balance_of(charlie).is_none());
			assert_eq!(Balances::free_balance(alice), 1000);
		});
}

#[test]
fn claim_with_not_exist_old_pub_key() {
	let (pair, charlie) = SubAccounts::Charlie.to_pair();
	let alice: H160 = EthAccounts::Alice.into();

	ExtBuilder::default().build().execute_with(|| {
		let message = ClaimMessage::new(42, &charlie, &alice);
		let sig = pair.sign(&blake2_256(&message.raw_bytes())[..]);

		assert_err!(
			AccountMigration::claim_to(RuntimeOrigin::none(), 42, charlie.clone(), alice, sig),
			Error::<TestRuntime>::AccountNotExist
		);
	});
}

#[test]
fn claim_to_existed_account() {
	let (pair, bogus) = SubAccounts::Bogus.to_pair();
	let bob: H160 = EthAccounts::Bob.into();

	ExtBuilder::default()
		.with_migrated_accounts(vec![(bogus.clone(), 1000)])
		.with_balances(vec![(bob, 500)])
		.build()
		.execute_with(|| {
			let message = ClaimMessage::new(42, &bogus, &bob);
			let sig = pair.sign(&blake2_256(&message.raw_bytes())[..]);

			assert_eq!(AccountMigration::balance_of(bogus.clone()), Some(1000));
			assert_eq!(Balances::free_balance(bob), 500);
			assert_ok!(AccountMigration::claim_to(
				RuntimeOrigin::none(),
				42,
				bogus.clone(),
				bob,
				sig
			));
			assert!(AccountMigration::balance_of(bogus).is_none());
			assert_eq!(Balances::free_balance(bob), 1000 + 500);
		});
}

#[test]
fn claim_event() {
	let (pair, charlie) = SubAccounts::Charlie.to_pair();
	let alice: H160 = EthAccounts::Alice.into();

	ExtBuilder::default()
		.with_migrated_accounts(vec![(charlie.clone(), 1000)])
		.build()
		.execute_with(|| {
			let message = ClaimMessage::new(42, &charlie, &alice);
			let sig = pair.sign(&blake2_256(&message.raw_bytes())[..]);

			assert_ok!(AccountMigration::claim_to(
				RuntimeOrigin::none(),
				42,
				charlie.clone(),
				alice,
				sig
			));
			System::assert_has_event(RuntimeEvent::AccountMigration(crate::Event::Claim {
				old_pub_key: charlie,
				new_pub_key: alice,
				amount: 1000,
			}))
		});
}

#[test]
fn claim_pre_dispatch_with_invalid_chain_id() {
	let (pair, charlie) = SubAccounts::Charlie.to_pair();
	let alice: H160 = EthAccounts::Alice.into();

	ExtBuilder::default()
		.with_migrated_accounts(vec![(charlie.clone(), 1000)])
		.build()
		.execute_with(|| {
			let message = ClaimMessage::new(42, &charlie, &alice);
			let sig = pair.sign(&blake2_256(&message.raw_bytes())[..]);

			let call = crate::Call::claim_to {
				chain_id: 43, // The correct chain id is 42
				old_pub_key: charlie.clone(),
				new_pub_key: alice,
				sig,
			};
			assert_err!(
				AccountMigration::pre_dispatch(&call),
				TransactionValidityError::Invalid(InvalidTransaction::BadProof)
			);
		});
}

#[test]
fn claim_pre_dispatch_with_invalid_old_pub_key() {
	let (pair, charlie) = SubAccounts::Charlie.to_pair();
	let alice: H160 = EthAccounts::Alice.into();

	ExtBuilder::default().with_migrated_accounts(vec![]).build().execute_with(|| {
		let message = ClaimMessage::new(42, &charlie, &alice);
		let sig = pair.sign(&blake2_256(&message.raw_bytes())[..]);

		let call = crate::Call::claim_to {
			chain_id: 42,
			old_pub_key: charlie.clone(),
			new_pub_key: alice,
			sig,
		};
		assert_err!(
			AccountMigration::pre_dispatch(&call),
			TransactionValidityError::Invalid(InvalidTransaction::BadSigner)
		);
	});
}

#[test]
fn claim_pre_dispatch_with_invalid_signature() {
	let (_, charlie) = SubAccounts::Charlie.to_pair();
	let (bogus_pair, _) = SubAccounts::Bogus.to_pair();
	let alice: H160 = EthAccounts::Alice.into();

	ExtBuilder::default()
		.with_migrated_accounts(vec![(charlie.clone(), 1000)])
		.build()
		.execute_with(|| {
			let message = ClaimMessage::new(42, &charlie, &alice);
			let sig = bogus_pair.sign(&blake2_256(&message.raw_bytes())[..]);

			let call = crate::Call::claim_to {
				chain_id: 42,
				old_pub_key: charlie.clone(),
				new_pub_key: alice,
				sig,
			};
			assert_err!(
				AccountMigration::pre_dispatch(&call),
				TransactionValidityError::Invalid(InvalidTransaction::BadSigner)
			);
		});
}
