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

//! # Darwinia asset limit pallet
//! Please note that this pallet is only for foreign assets.

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

		/// Origin that is allowed to create and modify asset information for local assets
		type LimitModifierOrigin: EnsureOrigin<Self::RuntimeOrigin>;
	}

    #[allow(missing_docs)]
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
        /// New limit is set or old limit is updated.
		AssetLimitChanged { asset_location: T::ForeignAssetType, limit: dc_types::Balance },
	}

	#[allow(missing_docs)]
	#[pallet::error]
	pub enum Error<T> {
        AssetDoesNotExist
    }

    /// Stores the asset limit for foreign assets.
	#[pallet::storage]
	#[pallet::getter(fn foreign_asset_limit)]
	pub type ForeignAssetLimit<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ForeignAssetType, dc_types::Balance>;

    
	#[pallet::call]
	impl<T: Config> Pallet<T> {
        /// Change the asset limit for a given asset location.
		#[pallet::call_index(0)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
		pub fn set_foreign_asset_limit(
			origin: OriginFor<T>,
			asset_location: T::ForeignAssetType,
			limit: dc_types::Balance,
		) -> DispatchResult {
			T::LimitModifierOrigin::ensure_origin(origin)?;

            ensure!(
				pallet_asset_manager::AssetTypeId::<T>::get(&asset_location).is_some(),
				Error::<T>::AssetDoesNotExist
			);

			ForeignAssetLimit::<T>::insert(&asset_location, &limit);

			Self::deposit_event(Event::AssetLimitChanged { asset_location, limit });
			Ok(())
		}
	}
}
