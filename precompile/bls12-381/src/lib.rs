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

// core
use core::marker::PhantomData;
// crates.io
use milagro_bls::{AggregatePublicKey, AggregateSignature, PublicKey, Signature};
// moonbeam
use precompile_utils::prelude::*;
// substrate
use sp_std::prelude::*;

pub(crate) const VERIFY_ESTIMATED_COST: u64 = 100_000;
pub struct BLS12381<T>(PhantomData<T>);

#[precompile_utils::precompile]
impl<Runtime: pallet_evm::Config> BLS12381<Runtime> {
	#[precompile::public("fast_aggregate_verify(bytes[],bytes,bytes)")]
	#[precompile::view]
	fn state_storage_at(
		handle: &mut impl PrecompileHandle,
		pubkeys: Vec<UnboundedBytes>,
		message: UnboundedBytes,
		signature: UnboundedBytes,
	) -> EvmResult<bool> {
		handle.record_cost(VERIFY_ESTIMATED_COST)?;

		let sig =
			Signature::from_bytes(signature.as_bytes()).map_err(|_| revert("Invalid signature"))?;
		let agg_sig = AggregateSignature::from_signature(&sig);

		let public_keys: Result<Vec<PublicKey>, _> =
			pubkeys.into_iter().map(|k| PublicKey::from_bytes(k.as_bytes())).collect();
		let Ok(keys) = public_keys else {
            return Err(revert("Invalid pubkeys"));
        };

		let agg_pub_key =
			AggregatePublicKey::into_aggregate(&keys).map_err(|_| revert("Invalid aggregate"))?;
		Ok(agg_sig.fast_aggregate_verify_pre_aggregated(message.as_bytes(), &agg_pub_key))
	}
}
