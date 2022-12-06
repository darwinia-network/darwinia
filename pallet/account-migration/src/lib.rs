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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod test;

/// Type alias for currency AccountId.
type AccountIdOf<R> = <R as frame_system::pallet::Config>::AccountId;
/// Type alias for currency balance.
type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

// darwinia
use dc_primitives::Balance;
// substrate
use frame_support::traits::Currency;
#[cfg(feature = "std")]
use frame_support::traits::GenesisBuild;
use sp_core::{
	crypto::ByteArray,
	sr25519::{Public, Signature},
	H160,
};
use sp_io::hashing::blake2_256;
use sp_runtime::{traits::Verify, AccountId32};
use sp_std::vec::Vec;

pub use pallet::*;
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Currency type for the runtime.
		type Currency: Currency<Self::AccountId>;
	}

	// Store the migrated balance snapshot for the darwinia-1.0 chain state.
	#[pallet::storage]
	#[pallet::getter(fn balance_of)]
	pub(super) type Balances<T> = StorageMap<_, Blake2_128Concat, AccountId32, Balance>;

	#[pallet::error]
	pub enum Error<T> {
		/// This account does not exist in the darwinia 1.0 chain state.
		AccountNotExist,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event {
		/// Claim to the new account id.
		Claim { old_pub_key: AccountId32, new_pub_key: H160, amount: Balance },
	}

	#[pallet::genesis_config]
	#[cfg_attr(feature = "std", derive(Default))]
	pub struct GenesisConfig {
		pub migrated_accounts: Vec<(AccountId32, Balance)>,
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			self.migrated_accounts.iter().for_each(|(account, amount)| {
				Balances::<T>::insert(account, amount);
			});
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		AccountIdOf<T>: From<H160>,
		BalanceOf<T>: From<Balance>,
	{
		// since signature and chain_id verification is done in `validate_unsigned`
		// we can skip doing it here again.
		// TODO: update weight
		#[pallet::weight(0)]
		pub fn claim_to(
			origin: OriginFor<T>,
			_chain_id: u64,
			old_pub_key: AccountId32,
			new_pub_key: H160,
			_sig: Signature,
		) -> DispatchResult {
			ensure_none(origin)?;

			let Some(amount) = Balances::<T>::take(&old_pub_key) else  {
				return Err(Error::<T>::AccountNotExist.into());
			};

			<T as pallet::Config>::Currency::deposit_creating(&new_pub_key.into(), amount.into());
			Self::deposit_event(Event::Claim { old_pub_key, new_pub_key, amount });

			Ok(())
		}
	}
	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T>
	where
		AccountIdOf<T>: From<H160>,
		BalanceOf<T>: From<Balance>,
	{
		type Call = Call<T>;

		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			let Call::claim_to { chain_id, old_pub_key, new_pub_key, sig } = call else {
				return InvalidTransaction::Call.into();
			};

			if *chain_id != <T as pallet_evm::Config>::ChainId::get() {
				return InvalidTransaction::BadProof.into();
			}
			if !Balances::<T>::contains_key(old_pub_key) {
				return InvalidTransaction::BadSigner.into();
			}

			let message = ClaimMessage::new(
				<T as pallet_evm::Config>::ChainId::get(),
				old_pub_key,
				new_pub_key,
			);
			if let Ok(signer) = Public::from_slice(old_pub_key.as_ref()) {
				let is_valid = sig.verify(&blake2_256(&message.raw_bytes())[..], &signer);

				if is_valid {
					return ValidTransaction::with_tag_prefix("MigrateClaim")
						.priority(TransactionPriority::max_value())
						.propagate(true)
						.build();
				}
			}
			InvalidTransaction::BadSigner.into()
		}
	}
}

/// ClaimMessage is the metadata that needs to be signed when the user invokes claim dispatch.
///
/// It consists of three parts, namely the chain_id, the AccountId32 account for darwinia 1.0, and
/// the H160 account for darwinia 2.0.
pub struct ClaimMessage<'m> {
	pub chain_id: u64,
	pub old_pub_key: &'m AccountId32,
	pub new_pub_key: &'m H160,
}

impl<'m> ClaimMessage<'m> {
	fn new(chain_id: u64, old_pub_key: &'m AccountId32, new_pub_key: &'m H160) -> Self {
		Self { chain_id, old_pub_key, new_pub_key }
	}

	fn raw_bytes(&self) -> Vec<u8> {
		let mut result = Vec::new();
		result.extend_from_slice(&self.chain_id.to_be_bytes());
		result.extend_from_slice(self.old_pub_key.as_slice());
		result.extend_from_slice(self.new_pub_key.as_bytes());
		result
	}
}
