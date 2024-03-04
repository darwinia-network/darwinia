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

mod bls;
use bls::{hash_to_curve_g2, PublicKey, Signature};

// core
use core::marker::PhantomData;
// frontier
use pallet_evm::GasWeightMapping;
// moonbeam
use precompile_utils::prelude::*;
// substrate
use frame_support::{ensure, weights::Weight};
use sp_std::prelude::*;

/// The BLS verification is a computationally intensive process. Normally, it consumes a lot of
/// block weight according to our benchmark test. Tested verifying of 512 public keys signature on
/// the `AMD Ryzen 7 5700G`,  this precompile consumed at least 117_954_459_000 weight. So we give
/// them more than that to ensure there is enough time for other machine types.
const BLS_WEIGHT: u64 = 150_000_000_000;

pub struct BLS12381<T>(PhantomData<T>);

#[precompile_utils::precompile]
impl<Runtime: pallet_evm::Config> BLS12381<Runtime> {
	/// FastAggregateVerify
	///
	/// Verifies an aggregate_signature against a list of pub_keys.
	/// pub_keys must be trusted the origin of the serialization
	/// precompile do not check the keys is valid
	/// see more: https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-bls-signature-04#section-2.5
	#[precompile::public("fast_aggregate_verify(bytes[],bytes,bytes)")]
	#[precompile::view]
	fn fast_aggregate_verify(
		handle: &mut impl PrecompileHandle,
		pub_keys: Vec<UnboundedBytes>,
		message: UnboundedBytes,
		signature: UnboundedBytes,
	) -> EvmResult<bool> {
		handle.record_cost(<Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			Weight::from_parts(BLS_WEIGHT, 0),
		))?;

		ensure!(pub_keys.len() <= 512, revert("Too many pub keys"));

		let asig =
			Signature::from_bytes(signature.as_bytes()).map_err(|_| revert("Invalid signature"))?;
		let pub_keys: Result<Vec<PublicKey>, _> =
			pub_keys.into_iter().map(|k| PublicKey::from_bytes(k.as_bytes())).collect();
		let Ok(pks) = pub_keys else {
            return Err(revert("Invalid pub keys"));
        };

		let apk = PublicKey::aggregate(pks);
		let msg = hash_to_curve_g2(message.as_bytes()).map_err(|_| revert("Invalid message"))?;
		Ok(apk.verify(&asig, &msg))
	}
}
