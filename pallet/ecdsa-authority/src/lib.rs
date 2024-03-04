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

//! # Relay Authorities Module
//! Works with https://github.com/darwinia-network/darwinia-messages-sol/pull/217

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::type_complexity)]
#![deny(missing_docs)]
#![deny(unused_crate_dependencies)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod test_utils;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod primitives;
use primitives::*;

mod weights;
pub use weights::WeightInfo;

// crates.io
use ethabi::Token;
// darwinia
use dc_primitives::AccountId;
// substrate
use frame_support::{pallet_prelude::*, DefaultNoBound};
use frame_system::pallet_prelude::*;
use sp_runtime::{
	traits::{SaturatedConversion, Zero},
	Perbill, Saturating,
};
use sp_std::prelude::*;

#[frame_support::pallet]
pub mod pallet {
	// darwinia
	use crate::*;

	#[pallet::config]
	pub trait Config: frame_system::Config<AccountId = AccountId> {
		/// Override the [`frame_system::Config::RuntimeEvent`].
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight.
		type WeightInfo: WeightInfo;

		/// The maximum number of authorities.
		#[pallet::constant]
		type MaxAuthorities: Get<u32>;
		/// Chain's ID, which is using for constructing the message. (follow EIP-712 SPEC)
		#[pallet::constant]
		type ChainId: Get<u64>;

		/// The signing threshold.
		///
		/// Once `signatures_count / authorities_count >= threshold`, we say the message is trusted.
		#[pallet::constant]
		type SignThreshold: Get<Perbill>;

		/// The interval of checking the message root.
		/// This must be shorter than [`Config::MaxPendingPeriod`].
		#[pallet::constant]
		type SyncInterval: Get<BlockNumberFor<Self>>;

		/// How long should we wait for the message root to be signed.
		///
		/// If the collecting new message root signatures process takes more than
		/// `MaxPendingPeriod`, we will drop the root. And update the root with a new one.
		#[pallet::constant]
		type MaxPendingPeriod: Get<BlockNumberFor<Self>>;

		/// The Darwinia message root.
		///
		/// If it changed, it means there are some new messages which are waiting for relaying.
		type MessageRoot: Get<Option<Hash>>;
	}

	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Authorities changed. Collecting authorities change signatures.
		CollectingAuthoritiesChangeSignatures { message: Hash },
		/// Collected enough authorities change signatures.
		CollectedEnoughAuthoritiesChangeSignatures {
			operation: Operation<T::AccountId>,
			threshold: Option<u32>,
			message: Hash,
			signatures: Vec<(T::AccountId, Signature)>,
		},
		/// New message root found. Collecting new message root signatures.
		CollectingNewMessageRootSignatures { message: Hash },
		/// Collected enough new message root signatures.
		CollectedEnoughNewMessageRootSignatures {
			commitment: Commitment<BlockNumberFor<T>>,
			message: Hash,
			signatures: Vec<(T::AccountId, Signature)>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The authority is already existed.
		AuthorityExisted,
		/// Too many authorities.
		TooManyAuthorities,
		/// This is not an authority.
		NotAuthority,
		/// Require at least one authority. Not allow to decrease below one.
		AtLeastOneAuthority,
		/// Currently, the authorities is changing.
		OnAuthoritiesChange,
		/// Didn't find any authorities changes to sign.
		NoAuthoritiesChange,
		/// Didn't find any new message root to sign.
		NoNewMessageRoot,
		/// Failed to verify the signature.
		BadSignature,
		/// This authority had already finished his duty.
		AlreadySubmitted,
	}

	/// The current active authorities.
	#[pallet::storage]
	#[pallet::getter(fn authorities)]
	pub type Authorities<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxAuthorities>, ValueQuery>;

	/// The incoming authorities.
	#[pallet::storage]
	#[pallet::getter(fn next_authorities)]
	pub type NextAuthorities<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxAuthorities>, ValueQuery>;

	/// The nonce of the current active authorities. AKA term/session/era.
	#[pallet::storage]
	#[pallet::getter(fn nonce)]
	pub type Nonce<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// The authorities change waiting for signing.
	#[pallet::storage]
	#[pallet::getter(fn authorities_change_to_sign)]
	pub type AuthoritiesChangeToSign<T: Config> =
		StorageValue<_, AuthoritiesChangeSigned<T::MaxAuthorities>>;

	/// The incoming message root waiting for signing.
	#[pallet::storage]
	#[pallet::getter(fn message_root_to_sign)]
	pub type MessageRootToSign<T: Config> =
		StorageValue<_, MessageRootSigned<BlockNumberFor<T>, T::MaxAuthorities>>;

	#[derive(DefaultNoBound)]
	#[pallet::genesis_config]
	pub struct GenesisConfig<T>
	where
		T: Config,
	{
		/// The genesis authorities.
		pub authorities: Vec<T::AccountId>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			<Authorities<T>>::put(BoundedVec::try_from(self.authorities.clone()).unwrap());
			<NextAuthorities<T>>::put(BoundedVec::try_from(self.authorities.clone()).unwrap());
		}
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(now: BlockNumberFor<T>) -> Weight {
			if (now % T::SyncInterval::get()).is_zero() {
				let now = now.saturated_into();

				if let Some(message_root) = Self::try_update_message_root(now, false) {
					Self::on_new_message_root(now, message_root);
				}

				T::WeightInfo::on_initialize()
			} else {
				Default::default()
			}
		}
	}
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add a authority and trigger `on_authorities_change`.
		///
		/// Not allow to call while authorities is changing.
		/// This will insert new authority into the index 0 of authorities.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::add_authority())]
		pub fn add_authority(origin: OriginFor<T>, new: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;

			Self::ensure_not_on_authorities_change()?;

			let authorities_count = <NextAuthorities<T>>::try_mutate(|authorities| {
				if authorities.contains(&new) {
					return Err(<Error<T>>::AuthorityExisted)?;
				}

				authorities.try_insert(0, new).map_err(|_| <Error<T>>::TooManyAuthorities)?;

				Ok::<_, DispatchError>(authorities.len() as u32)
			})?;

			Self::on_authorities_change(Operation::AddMember { new }, authorities_count);

			Ok(())
		}

		/// Remove a authority and trigger `on_authorities_change`.
		///
		/// Not allow to call while authorities is changing.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::remove_authority())]
		pub fn remove_authority(origin: OriginFor<T>, old: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;

			Self::ensure_not_on_authorities_change()?;

			let (authorities_count, pre) = <NextAuthorities<T>>::try_mutate(|authorities| {
				let i =
					authorities.iter().position(|a| a == &old).ok_or(<Error<T>>::NotAuthority)?;

				if authorities.len() == 1 {
					return Err(<Error<T>>::AtLeastOneAuthority)?;
				}

				authorities.remove(i);

				Ok::<_, DispatchError>((
					authorities.len() as u32,
					if i == 0 { AUTHORITY_SENTINEL.into() } else { authorities[i - 1] },
				))
			})?;

			Self::on_authorities_change(Operation::RemoveMember { pre, old }, authorities_count);

			Ok(())
		}

		/// Swap the old authority with the new authority and trigger `on_authorities_change`.
		///
		/// Not allow to call while authorities is changing.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::swap_authority())]
		pub fn swap_authority(
			origin: OriginFor<T>,
			old: T::AccountId,
			new: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;

			Self::ensure_not_on_authorities_change()?;

			let (authorities_count, pre) = <NextAuthorities<T>>::try_mutate(|authorities| {
				let i =
					authorities.iter().position(|a| a == &old).ok_or(<Error<T>>::NotAuthority)?;

				authorities[i] = new;

				Ok::<_, DispatchError>((
					authorities.len() as u32,
					if i == 0 { AUTHORITY_SENTINEL.into() } else { authorities[i - 1] },
				))
			})?;

			Self::on_authorities_change(
				Operation::SwapMembers { pre, old, new },
				authorities_count,
			);

			Ok(())
		}

		/// Submit the authorities change signature.
		///
		/// Free to submit the first-correct signature.
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::submit_authorities_change_signature())]
		pub fn submit_authorities_change_signature(
			origin: OriginFor<T>,
			signature: Signature,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let authorities = Self::ensure_authority(&who)?;
			let mut authorities_change_signed =
				<AuthoritiesChangeToSign<T>>::get().ok_or(<Error<T>>::NoAuthoritiesChange)?;

			Self::ensure_not_submitted(&who, &authorities_change_signed.signatures)?;

			ensure!(
				Sign::verify_signature(&signature.0, &authorities_change_signed.message.0, &who.0),
				<Error<T>>::BadSignature
			);

			authorities_change_signed
				.signatures
				.try_push((who, signature))
				.map_err(|_| <Error<T>>::TooManyAuthorities)?;

			if Self::check_threshold(
				authorities_change_signed.signatures.len() as _,
				authorities.len() as _,
			) {
				Self::apply_next_authorities();
				Self::deposit_event(Event::<T>::CollectedEnoughAuthoritiesChangeSignatures {
					operation: authorities_change_signed.operation,
					threshold: authorities_change_signed.threshold,
					message: authorities_change_signed.message,
					signatures: authorities_change_signed.signatures.to_vec(),
				});

				let now = <frame_system::Pallet<T>>::block_number().saturated_into();

				if let Some(message_root) = Self::try_update_message_root(now, true) {
					Self::on_new_message_root(now, message_root);
				}
			} else {
				<AuthoritiesChangeToSign<T>>::put(authorities_change_signed);
			}

			Ok(Pays::No.into())
		}

		/// Submit the new message root signature.
		///
		/// Free to submit the first-correct signature.
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::submit_new_message_root_signature())]
		pub fn submit_new_message_root_signature(
			origin: OriginFor<T>,
			signature: Signature,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let authorities = Self::ensure_authority(&who)?;
			let mut message_root_signed =
				<MessageRootToSign<T>>::get().ok_or(<Error<T>>::NoNewMessageRoot)?;

			Self::ensure_not_submitted(&who, &message_root_signed.signatures)?;

			ensure!(
				Sign::verify_signature(&signature.0, &message_root_signed.message.0, &who.0),
				<Error<T>>::BadSignature
			);

			message_root_signed
				.signatures
				.try_push((who, signature))
				.map_err(|_| <Error<T>>::TooManyAuthorities)?;

			if Self::check_threshold(
				message_root_signed.signatures.len() as _,
				authorities.len() as _,
			) {
				message_root_signed.authorized = true;

				Self::deposit_event(Event::<T>::CollectedEnoughNewMessageRootSignatures {
					commitment: message_root_signed.commitment.clone(),
					message: message_root_signed.message,
					signatures: message_root_signed.signatures.to_vec(),
				});
			}

			<MessageRootToSign<T>>::put(message_root_signed);

			Ok(Pays::No.into())
		}
	}
	impl<T: Config> Pallet<T> {
		fn ensure_authority(
			address: &T::AccountId,
		) -> Result<BoundedVec<T::AccountId, T::MaxAuthorities>, DispatchError> {
			let authorities = <Authorities<T>>::get();

			ensure!(authorities.iter().any(|a| a == address), <Error<T>>::NotAuthority);

			Ok(authorities)
		}

		fn ensure_not_on_authorities_change() -> DispatchResult {
			ensure!(!<AuthoritiesChangeToSign<T>>::exists(), <Error<T>>::OnAuthoritiesChange);

			Ok(())
		}

		fn ensure_not_submitted(
			who: &T::AccountId,
			collected: &[(T::AccountId, Signature)],
		) -> DispatchResult {
			ensure!(!collected.iter().any(|(a, _)| a == who), <Error<T>>::AlreadySubmitted);

			Ok(())
		}

		pub(crate) fn calculate_threshold(x: u32) -> u32 {
			T::SignThreshold::get().mul_ceil(x)
		}

		fn on_authorities_change(operation: Operation<T::AccountId>, authorities_count: u32) {
			let (authorities_changes, new_threshold) = {
				match operation {
					Operation::AddMember { new } => {
						let new_threshold = Self::calculate_threshold(authorities_count);

						(
							ethabi::encode(&[
								Token::Address(new.0.into()),
								Token::Uint(new_threshold.into()),
							]),
							Some(new_threshold),
						)
					},
					Operation::RemoveMember { pre, old } => {
						let new_threshold = Self::calculate_threshold(authorities_count);

						(
							ethabi::encode(&[
								Token::Address(pre.0.into()),
								Token::Address(old.0.into()),
								Token::Uint(new_threshold.into()),
							]),
							Some(new_threshold),
						)
					},
					Operation::SwapMembers { pre, old, new } => (
						ethabi::encode(&[
							Token::Address(pre.0.into()),
							Token::Address(old.0.into()),
							Token::Address(new.0.into()),
						]),
						None,
					),
				}
			};
			let message = Sign::signable_message(
				T::ChainId::get(),
				T::Version::get().spec_name.as_ref(),
				&ethabi::encode(&[
					Token::FixedBytes(RELAY_TYPE_HASH.into()),
					Token::FixedBytes(operation.id().into()),
					Token::Bytes(authorities_changes),
					Token::Uint(<Nonce<T>>::get().into()),
				]),
			);

			<AuthoritiesChangeToSign<T>>::put(AuthoritiesChangeSigned {
				operation,
				threshold: new_threshold,
				message,
				signatures: Default::default(),
			});

			Self::deposit_event(Event::<T>::CollectingAuthoritiesChangeSignatures { message });
		}

		fn check_threshold(p: u32, q: u32) -> bool {
			Perbill::from_rational(p, q) >= T::SignThreshold::get()
		}

		pub(crate) fn apply_next_authorities() {
			<AuthoritiesChangeToSign<T>>::kill();
			<Authorities<T>>::put(<NextAuthorities<T>>::get());
			<Nonce<T>>::mutate(|nonce| *nonce += 1);
		}

		fn try_update_message_root(at: BlockNumberFor<T>, force: bool) -> Option<Hash> {
			// Not allow to relay the messages if the new authorities set is not verified.
			if Self::ensure_not_on_authorities_change().is_err() {
				return None;
			}

			let new_message_root = if cfg!(all(feature = "runtime-benchmarks", not(test))) {
				Default::default()
			} else {
				T::MessageRoot::get()?
			};

			if force {
				return Some(new_message_root);
			}

			if let Some(message_root_signed) = <MessageRootToSign<T>>::get() {
				// If there is a new root.
				if new_message_root != message_root_signed.commitment.message_root {
					// If the root is already authorized, then we'll consider updating it.
					if message_root_signed.authorized {
						return Some(new_message_root);
					}

					// Else the previous root is still under signing process.
					//
					// Update the root with a new one if exceed the max pending period.
					// Also update the recorded time.
					if at.saturating_sub(message_root_signed.commitment.block_number)
						>= T::MaxPendingPeriod::get()
					{
						return Some(new_message_root);
					}
				}

				None
			} else {
				Some(new_message_root)
			}
		}

		fn on_new_message_root(at: BlockNumberFor<T>, message_root: Hash) {
			let commitment =
				Commitment { block_number: at, message_root, nonce: <Nonce<T>>::get() };
			let message = Sign::signable_message(
				T::ChainId::get(),
				T::Version::get().spec_name.as_ref(),
				&ethabi::encode(&[
					Token::FixedBytes(COMMIT_TYPE_HASH.into()),
					// Be careful to use this if your chain's block number is larger than
					// `u32::MAX`.
					Token::Uint(commitment.block_number.saturated_into::<u32>().into()),
					Token::FixedBytes(commitment.message_root.as_ref().into()),
					Token::Uint(commitment.nonce.into()),
				]),
			);

			<MessageRootToSign<T>>::put(MessageRootSigned {
				commitment,
				message,
				signatures: Default::default(),
				authorized: false,
			});

			Self::deposit_event(Event::<T>::CollectingNewMessageRootSignatures { message });
		}
	}
}
pub use pallet::*;
