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

//! # Darwinia account migration pallet
//!
//! ## Overview
//!
//! Darwinia2 uses ECDSA as its signature algorithm instead of SR25519.
//! These two algorithm are not compatible.
//! Thus, an account migration is required.
//!
//! ## Technical detail
//!
//! Users must send an extrinsic themselves to migrate their account(s).
//! This extrinsic should be unsigned, the reason is the same as `pallet-claims`.
//! This extrinsic's payload must contain a signature to the new ECDSA address, signed by their
//! origin SR25519 key.
//!
//! This pallet will store all the account data from Darwinia1 and Darwinia Parachain.
//! This pallet's genesis will be write into the chain spec JSON directly.
//! The data will be processed off-chain(ly).
//! After the verification, simply perform a take & put operation.
//!
//! ```nocompile
//! user -> send extrinsic -> verify -> put(storages, ECDSA, take(storages, SR25519))
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod weights;
pub use weights::WeightInfo;

// darwinia
use darwinia_deposit::{Deposit, DepositId};
use darwinia_staking::Ledger;
use dc_primitives::{AccountId as AccountId20, AssetId, Balance, BlockNumber, Nonce};
// polkadot-sdk
use frame_support::{
	migration,
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement::AllowDeath},
	StorageHasher,
};
use frame_system::{pallet_prelude::*, AccountInfo};
use pallet_balances::AccountData;
use sp_core::{
	ed25519::{Public as Ep, Signature as Es},
	sr25519::{Public as Sp, Signature as Ss},
};
use sp_io::hashing;
use sp_runtime::{
	traits::{IdentityLookup, TrailingZeroInput, Verify},
	AccountId32,
};
use sp_std::prelude::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	pub(crate) const KTON_ID: AssetId = 1026;

	/// The migration destination was already taken by someone.
	pub(crate) const E_ACCOUNT_ALREADY_EXISTED: u8 = 0;
	/// The migration source was not exist.
	pub(crate) const E_ACCOUNT_NOT_FOUND: u8 = 1;
	/// Invalid signature.
	pub(crate) const E_INVALID_SIGNATURE: u8 = 2;
	/// Duplicative submission.
	pub(crate) const E_DUPLICATIVE_SUBMISSION: u8 = 3;
	/// The account is not a member of the multisig.
	pub(crate) const E_NOT_MULTISIG_MEMBER: u8 = 4;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config:
		frame_system::Config<
			Nonce = Nonce,
			AccountId = AccountId20,
			AccountData = AccountData<Balance>,
			Lookup = IdentityLookup<AccountId20>,
		> + pallet_assets::Config<Balance = Balance, AssetId = AssetId>
		+ pallet_balances::Config<Balance = Balance>
		+ darwinia_deposit::Config
		+ darwinia_staking::Config
	{
		/// Override the [`frame_system::Config::RuntimeEvent`].
		type RuntimeEvent: From<Event> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event {
		/// An account has been migrated.
		Migrated { from: AccountId32, to: AccountId20 },
		/// A new multisig account params was noted/recorded on-chain.
		NewMultisigParamsNoted { from: AccountId32, to: MultisigParams },
		/// A multisig account has been migrated.
		MultisigMigrated { from: AccountId32, detail: MultisigMigrationDetail },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Exceed maximum deposit count.
		ExceedMaxDeposits,
		/// The migration destination was already taken by someone.
		AccountAlreadyExisted,
	}

	/// [`frame_system::Account`] data.
	///
	/// <https://github.dev/paritytech/substrate/blob/19162e43be45817b44c7d48e50d03f074f60fbf4/frame/system/src/lib.rs#L545>
	#[pallet::storage]
	#[pallet::getter(fn account_of)]
	pub type Accounts<T: Config> =
		StorageMap<_, Blake2_128Concat, AccountId32, AccountInfo<Nonce, AccountData<Balance>>>;

	/// [`pallet_asset::AssetAccount`] data.
	///
	/// https://github.dev/paritytech/substrate/blob/polkadot-v0.9.30/frame/assets/src/types.rs#L115
	#[pallet::storage]
	#[pallet::getter(fn kton_account_of)]
	pub type KtonAccounts<T: Config> = StorageMap<_, Blake2_128Concat, AccountId32, AssetAccount>;

	/// [`darwinia_deposit::Deposits`] data.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn deposit_of)]
	pub type Deposits<T: Config> = StorageMap<_, Blake2_128Concat, AccountId32, Vec<Deposit>>;

	/// [`darwinia_staking::migration::v2::OldLedger`] data.
	#[pallet::storage]
	#[pallet::getter(fn ledger_of)]
	pub type Ledgers<T: Config> = StorageMap<_, Blake2_128Concat, AccountId32, OldLedger>;

	/// Multisig migration caches.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn multisig_of)]
	pub type Multisigs<T: Config> = StorageMap<_, Identity, AccountId32, MultisigMigrationDetail>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Migrate all the account data under the `from` to `to`.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::migrate())]
		pub fn migrate(
			origin: OriginFor<T>,
			from: AccountId32,
			to: AccountId20,
			_signature: Signature,
		) -> DispatchResult {
			ensure_none(origin)?;

			Self::migrate_inner(&from, &to)?;
			Self::deposit_event(Event::Migrated { from, to });

			Ok(())
		}

		/// Similar to `migrate` but for multisig accounts.
		///
		/// The `_signature` should be provided by `who`.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::migrate_multisig(
			others.len() as _,
			*threshold as _,
			new_multisig_params.as_ref().map(|p| p.members.len()).unwrap_or_default() as _
		))]
		pub fn migrate_multisig(
			origin: OriginFor<T>,
			submitter: AccountId32,
			others: Vec<AccountId32>,
			threshold: u16,
			to: AccountId20,
			_signature: Signature,
			new_multisig_params: Option<MultisigParams>,
		) -> DispatchResult {
			ensure_none(origin)?;

			let (members, from) = multisig_of(submitter.clone(), others, threshold);
			let mut members = members.into_iter().map(|m| (m, false)).collect::<Vec<_>>();

			// Set the status to `true`.
			//
			// Because the `_signature` was already been verified in `pre_dispatch`.
			members
				.iter_mut()
				.find(|(who, _)| who == &submitter)
				.expect("[pallet::account-migration] `who` must be existed; qed")
				.1 = true;

			let detail = MultisigMigrationDetail { to, members, threshold };

			if threshold < 2 {
				Self::migrate_inner(&from, &to)?;

				Self::deposit_event(Event::MultisigMigrated { from: from.clone(), detail });
			} else {
				<Multisigs<T>>::insert(&from, detail);
			}

			if let Some(to) = new_multisig_params {
				Self::deposit_event(Event::NewMultisigParamsNoted { from, to });
			}

			Ok(())
		}

		/// To complete the pending multisig migration.
		///
		/// The `_signature` should be provided by `submitter`.
		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::complete_multisig_migration().saturating_add(<T as Config>::WeightInfo::migrate()))]
		pub fn complete_multisig_migration(
			origin: OriginFor<T>,
			multisig: AccountId32,
			submitter: AccountId32,
			_signature: Signature,
		) -> DispatchResult {
			ensure_none(origin)?;

			let from = multisig;
			let mut detail = <Multisigs<T>>::take(&from)
				.expect("[pallet::account-migration] already checked in `pre_dispatch`; qed");

			// Kill the storage, if the `to` was created during the migration.
			//
			// Require to redo the `migrate_multisig` operation.
			if <frame_system::Account<T>>::contains_key(detail.to) {
				Err(<Error<T>>::AccountAlreadyExisted)?;
			}

			// Set the status to `true`.
			//
			// Because the `_signature` was already been verified in `pre_dispatch`.
			detail
				.members
				.iter_mut()
				.find(|(who, _)| who == &submitter)
				.expect("[pallet::account-migration] already checked in `pre_dispatch`; qed")
				.1 = true;

			if detail.members.iter().fold(0, |acc, (_, ok)| if *ok { acc + 1 } else { acc })
				>= detail.threshold
			{
				Self::migrate_inner(&from, &detail.to)?;

				Self::deposit_event(Event::MultisigMigrated { from, detail });
			} else {
				<Multisigs<T>>::insert(from, detail);
			}

			Ok(())
		}
	}
	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			match call {
				Call::migrate { from, to, signature } => {
					Self::pre_check_existing(from, to)?;

					Self::pre_check_signature(from, to, signature)
				},
				Call::migrate_multisig { submitter, others, threshold, to, signature, .. } => {
					let (_, multisig) =
						multisig_of(submitter.to_owned(), others.to_owned(), *threshold);

					Self::pre_check_existing(&multisig, to)?;
					Self::pre_check_duplicative(&multisig)?;

					Self::pre_check_signature(submitter, to, signature)
				},
				Call::complete_multisig_migration { multisig, submitter, signature } => {
					let Some(multisig_info) = <Multisigs<T>>::get(multisig) else {
						return InvalidTransaction::Custom(E_ACCOUNT_NOT_FOUND).into();
					};
					let mut is_member = false;

					for (who, ok) in multisig_info.members.iter() {
						if who == submitter {
							// Reject duplicative submission.
							if *ok {
								return InvalidTransaction::Custom(E_DUPLICATIVE_SUBMISSION).into();
							}

							is_member = true;

							break;
						}
					}

					if !is_member {
						return InvalidTransaction::Custom(E_NOT_MULTISIG_MEMBER).into();
					}

					Self::pre_check_signature(submitter, &multisig_info.to, signature)
				},
				_ => InvalidTransaction::Call.into(),
			}
		}
	}
	impl<T> Pallet<T>
	where
		T: Config,
	{
		fn pre_check_existing(
			from: &AccountId32,
			to: &AccountId20,
		) -> Result<(), TransactionValidityError> {
			if !<Accounts<T>>::contains_key(from) {
				Err(InvalidTransaction::Custom(E_ACCOUNT_NOT_FOUND))?;
			}
			if <frame_system::Account<T>>::contains_key(to) {
				Err(InvalidTransaction::Custom(E_ACCOUNT_ALREADY_EXISTED))?;
			}

			Ok(())
		}

		fn pre_check_duplicative(multisig: &AccountId32) -> Result<(), TransactionValidityError> {
			if <Multisigs<T>>::contains_key(multisig) {
				Err(InvalidTransaction::Custom(E_DUPLICATIVE_SUBMISSION))?
			} else {
				Ok(())
			}
		}

		fn pre_check_signature(
			from: &AccountId32,
			to: &AccountId20,
			signature: &Signature,
		) -> TransactionValidity {
			let message = signable_message(T::Version::get().spec_name.as_ref(), to);

			if verify_curve_25519_signature(from, &message, signature) {
				ValidTransaction::with_tag_prefix("account-migration")
					.and_provides(from)
					.priority(100)
					.longevity(TransactionLongevity::max_value())
					.propagate(true)
					.build()
			} else {
				InvalidTransaction::Custom(E_INVALID_SIGNATURE).into()
			}
		}

		fn migrate_inner(from: &AccountId32, to: &AccountId20) -> DispatchResult {
			let mut account = <Accounts<T>>::take(from)
				.expect("[pallet::account-migration] already checked in `pre_dispatch`; qed");

			account.data.free += account.data.reserved;
			account.data.reserved = 0;

			<frame_system::Account<T>>::insert(to, account);

			if let Some(a) = <KtonAccounts<T>>::take(from) {
				let encoded_kton_id = KTON_ID.encode();

				migration::put_storage_value(
					b"Assets",
					b"Account",
					&[
						Blake2_128Concat::hash(&encoded_kton_id),
						Blake2_128Concat::hash(&to.encode()),
					]
					.concat(),
					a,
				);

				// Update the asset's accounts and sufficients.
				if let Some(mut asset_details) = migration::take_storage_value::<AssetDetails>(
					b"Assets",
					b"Asset",
					&Blake2_128Concat::hash(&encoded_kton_id),
				) {
					asset_details.accounts += 1;
					asset_details.sufficients += 1;

					migration::put_storage_value(
						b"Assets",
						b"Asset",
						&Blake2_128Concat::hash(&encoded_kton_id),
						asset_details,
					);
				}
				if let Some(l) = <Ledgers<T>>::take(from) {
					if l.staked_ring > 0 {
						<pallet_balances::Pallet<T> as Currency<_>>::transfer(
							to,
							&darwinia_staking::account_id(),
							l.staked_ring,
							AllowDeath,
						)?;
					}

					if let Some(ds) = <Deposits<T>>::take(from) {
						<pallet_balances::Pallet<T> as Currency<_>>::transfer(
							to,
							&darwinia_deposit::account_id(),
							ds.iter().map(|d| d.value).sum(),
							AllowDeath,
						)?;
						<darwinia_deposit::Deposits<T>>::insert(
							to,
							BoundedVec::try_from(ds).map_err(|_| <Error<T>>::ExceedMaxDeposits)?,
						);
					}

					<darwinia_staking::Ledgers<T>>::insert(
						to,
						Ledger { ring: l.staked_ring, deposits: l.staked_deposits },
					);
				}
			}

			Ok(())
		}
	}
}
pub use pallet::*;

/// Raw signature.
pub(crate) type Signature = [u8; 64];

// Copy from <https://github.dev/paritytech/substrate/blob/polkadot-v0.9.30/frame/assets/src/types.rs#L115>.
// Due to its visibility.
#[allow(missing_docs)]
#[derive(PartialEq, Eq, Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub struct AssetAccount {
	balance: Balance,
	is_frozen: bool,
	reason: ExistenceReason,
	extra: (),
}
#[allow(missing_docs)]
#[derive(PartialEq, Eq, Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub(crate) enum ExistenceReason {
	#[codec(index = 0)]
	Consumer,
	#[codec(index = 1)]
	Sufficient,
	#[codec(index = 2)]
	DepositHeld(Balance),
	#[codec(index = 3)]
	DepositRefunded,
}
#[allow(missing_docs)]
#[derive(PartialEq, Eq, Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub(crate) struct AssetDetails {
	owner: AccountId20,
	issuer: AccountId20,
	admin: AccountId20,
	freezer: AccountId20,
	supply: Balance,
	deposit: Balance,
	min_balance: Balance,
	is_sufficient: bool,
	accounts: u32,
	sufficients: u32,
	approvals: u32,
	status: AssetStatus,
}
#[allow(missing_docs)]
#[derive(PartialEq, Eq, Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub(crate) enum AssetStatus {
	Live,
	Frozen,
	Destroying,
}

#[allow(missing_docs)]
#[derive(Default, PartialEq, Eq, Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub struct OldLedger {
	pub staked_ring: Balance,
	pub staked_kton: Balance,
	pub staked_deposits: BoundedVec<DepositId, ConstU32<512>>,
	pub unstaking_ring: BoundedVec<(Balance, BlockNumber), ConstU32<512>>,
	pub unstaking_kton: BoundedVec<(Balance, BlockNumber), ConstU32<512>>,
	pub unstaking_deposits: BoundedVec<(DepositId, BlockNumber), ConstU32<512>>,
}

#[allow(missing_docs)]
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct MultisigParams {
	address: AccountId20,
	members: Vec<AccountId20>,
	threshold: u16,
}

#[allow(missing_docs)]
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct MultisigMigrationDetail {
	to: AccountId20,
	members: Vec<(AccountId32, bool)>,
	threshold: u16,
}

/// Build a Darwinia account migration message.
pub fn signable_message(spec_name: &[u8], account_id_20: &AccountId20) -> Vec<u8> {
	[
		// https://github.com/polkadot-js/common/issues/1710
		b"<Bytes>I authorize the migration to ",
		// Ignore the EIP-55 here.
		//
		// Must call the `to_lowercase` on front end.
		array_bytes::bytes2hex("0x", account_id_20.0).as_bytes(),
		b", an unused address on ",
		spec_name,
		b". Sign this message to authorize using the Substrate key associated with the account on ",
		&spec_name[..spec_name.len() - 1],
		b" that you wish to migrate.</Bytes>",
	]
	.concat()
}

/// Verify the curve 25519 signatures.
pub(crate) fn verify_curve_25519_signature(
	public_key: &AccountId32,
	message: &[u8],
	signature: &Signature,
) -> bool {
	Ss(signature.to_owned()).verify(message, &Sp(public_key.to_owned().into()))
		|| Es(signature.to_owned()).verify(message, &Ep(public_key.to_owned().into()))
}

/// Calculate the multisig account.
pub(crate) fn multisig_of(
	who: AccountId32,
	others: Vec<AccountId32>,
	threshold: u16,
) -> (Vec<AccountId32>, AccountId32) {
	// https://github.com/paritytech/substrate/blob/3bc3742d5c0c5269353d7809d9f8f91104a93273/frame/multisig/src/lib.rs#L525
	fn multisig_of_inner(members: &[AccountId32], threshold: u16) -> AccountId32 {
		let entropy = (b"modlpy/utilisuba", members, threshold).using_encoded(hashing::blake2_256);

		Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref())).expect(
			"[pallet::account-migration] infinite length input; no invalid inputs for type; qed",
		)
	}

	let mut members = others;

	members.push(who);
	members.sort();

	let multisig = multisig_of_inner(&members, threshold);

	(members, multisig)
}
