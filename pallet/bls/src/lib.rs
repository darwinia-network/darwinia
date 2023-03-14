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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod mock;

mod bls;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use bls::{hash_to_curve_g2, PublicKey, Signature};
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		A,
		B,
		C,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn fast_aggregate_verify_latest(
			_origin: OriginFor<T>,
			message: Vec<u8>,
			pubkeys: Vec<Vec<u8>>,
			signature: Vec<u8>,
		) -> DispatchResult {
			let asig = Signature::from_bytes(&signature).map_err(|_| Error::<T>::A)?;
			let public_keys: Result<Vec<PublicKey>, _> =
				pubkeys.into_iter().map(|k| PublicKey::from_bytes(&k)).collect();
			let Ok(pks) = public_keys else {
            return Err(Error::<T>::B.into());
            };

			let apk = PublicKey::aggregate(pks);
			let msg = hash_to_curve_g2(&message).map_err(|_| Error::<T>::C)?;
			let result = apk.verify(&asig, &msg);
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn fast_aggregate_verify_before(
			_origin: OriginFor<T>,
			message: Vec<u8>,
			pubkeys: Vec<Vec<u8>>,
			signature: Vec<u8>,
		) -> DispatchResult {
			use milagro_bls::{AggregatePublicKey, AggregateSignature, PublicKey, Signature};

			let sig = Signature::from_bytes(&signature).map_err(|_| Error::<T>::A)?;
			let agg_sig = AggregateSignature::from_signature(&sig);

			let public_keys: Result<Vec<PublicKey>, _> =
				pubkeys.into_iter().map(|k| PublicKey::from_bytes(&k)).collect();
			let Ok(keys) = public_keys else {
            	return Err(Error::<T>::B.into());
        	};

			let agg_pub_key =
				AggregatePublicKey::into_aggregate(&keys).map_err(|_| Error::<T>::C)?;
			let result = agg_sig.fast_aggregate_verify_pre_aggregated(&message, &agg_pub_key);
			Ok(())
		}
	}
}

pub use pallet::*;
