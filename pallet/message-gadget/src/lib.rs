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

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unused_crate_dependencies)]

// core
use core::marker::PhantomData;
// frontier
use pallet_evm::Runner;
// substrate
use frame_support::{pallet_prelude::*, DefaultNoBound};
use frame_system::pallet_prelude::*;
use sp_core::{Get, H160, H256};
use sp_io::hashing;
use sp_std::prelude::*;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	// darwinia
	use crate::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::storage]
	#[pallet::getter(fn commitment_contract)]
	pub type CommitmentContract<T> = StorageValue<_, H160, ValueQuery>;

	#[derive(DefaultNoBound)]
	#[pallet::genesis_config]
	pub struct GenesisConfig<T> {
		pub commitment_contract: H160,
		_marker: PhantomData<T>,
	}
	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			<CommitmentContract<T>>::put(H160::default());
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight({0})]
		pub fn set_commitment_contract(
			origin: OriginFor<T>,
			commitment_contract: H160,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			<CommitmentContract<T>>::put(commitment_contract);

			Ok(().into())
		}
	}
}
pub use pallet::*;

pub struct MessageRootGetter<T>(PhantomData<T>);
impl<T> Get<Option<H256>> for MessageRootGetter<T>
where
	T: Config + pallet_evm::Config,
{
	fn get() -> Option<H256> {
		if let Ok(info) = <T as pallet_evm::Config>::Runner::call(
			H160::default(),
			<CommitmentContract<T>>::get(),
			hashing::keccak_256(b"commitment()")[..4].to_vec(),
			0.into(),
			1_000_000_000_000,
			None,
			None,
			None,
			Vec::new(),
			false,
			false,
			None,
			None,
			<T as pallet_evm::Config>::config(),
		) {
			let raw_message_root = info.value;
			if raw_message_root.len() != 32 {
				log::warn!(
					"[pallet::message-gadget] invalid raw message root: {:?}, return.",
					raw_message_root
				);

				return None;
			}
			return Some(H256::from_slice(&raw_message_root));
		}

		None
	}
}
