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

//! # Darwinia asset limit pallet
//! Please note that this pallet is only for foreign assets.
//! This pallet only stores the asset limit and does not actually prevent asset deposits.
//! Currently, we use the app to limit excessive amounts of deposits.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
#![deny(unused_crate_dependencies)]

#[frame_support::pallet]
pub mod pallet {
	// substrate
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_asset_manager::Config {
		/// Override the [`frame_system::Config::RuntimeEvent`].
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New limit is set or old limit is updated.
		AssetLimitChanged { asset_type: T::ForeignAssetType, units_limit: u128 },
	}

	#[allow(missing_docs)]
	#[pallet::error]
	pub enum Error<T> {
		/// Asset does not exist.
		AssetDoesNotExist,
	}

	/// Stores the asset limit for foreign assets.
	#[pallet::storage]
	#[pallet::getter(fn foreign_asset_limit)]
	pub type ForeignAssetLimit<T: Config> = StorageMap<_, Twox128, T::ForeignAssetType, u128>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Change the asset limit for a given foreign asset type.
		#[pallet::call_index(0)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
		pub fn set_foreign_asset_limit(
			origin: OriginFor<T>,
			asset_type: T::ForeignAssetType,
			units_limit: u128,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;

			ensure!(
				pallet_asset_manager::AssetTypeId::<T>::get(&asset_type).is_some(),
				Error::<T>::AssetDoesNotExist
			);

			ForeignAssetLimit::<T>::insert(&asset_type, units_limit);

			Self::deposit_event(Event::AssetLimitChanged { asset_type, units_limit });
			Ok(())
		}
	}
}
pub use pallet::*;
