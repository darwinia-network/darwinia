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

mod bls;
use bls::{hash_to_curve_g2, PublicKey, Signature};

// core
use core::marker::PhantomData;
// frontier
use pallet_evm::GasWeightMapping;
// moonbeam
use precompile_utils::prelude::*;
// substrate
use frame_support::weights::Weight;
use sp_std::prelude::*;

// pub(crate) const BLS_BENCHMARKED_WEIGHT: u64 = 117_954_459_000;
pub(crate) const BLS_ESTIMATED_COST: u64 = 100_000;
pub struct BLS12381<T>(PhantomData<T>);

#[precompile_utils::precompile]
impl<Runtime: pallet_evm::Config> BLS12381<Runtime> {
	#[precompile::public("fast_aggregate_verify(bytes[],bytes,bytes)")]
	#[precompile::view]
	fn fast_aggregate_verify(
		handle: &mut impl PrecompileHandle,
		pubkeys: Vec<UnboundedBytes>,
		message: UnboundedBytes,
		signature: UnboundedBytes,
	) -> EvmResult<bool> {
		frame_support::log::info!("bear: --- here in the precompile");
		// handle.record_cost(<Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
		// 	Weight::from_ref_time(BLS_BENCHMARKED_WEIGHT),
		// ))?;
		handle.record_cost(BLS_ESTIMATED_COST)?;

		let asig =
			Signature::from_bytes(signature.as_bytes()).map_err(|_| revert("Invalid signature"))?;
		frame_support::log::info!("bear: --- flag 1");
		let public_keys: Result<Vec<PublicKey>, _> =
			pubkeys.into_iter().map(|k| PublicKey::from_bytes(k.as_bytes())).collect();
			frame_support::log::info!("bear: --- flag 2");
		let Ok(pks) = public_keys else {
            return Err(revert("Invalid pubkeys"));
        };
		frame_support::log::info!("bear: --- flag 3");

		let apk = PublicKey::aggregate(pks);
		let msg = hash_to_curve_g2(message.as_bytes()).map_err(|_| revert("Invalid message"))?;
		frame_support::log::info!("bear: --- flag 4");
		let result = apk.verify(&asig, &msg);
		frame_support::log::info!("bear: --- flag 5, {:?}", result);
		Ok(result)
	}
}
