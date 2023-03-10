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
// moonbeam
use precompile_utils::prelude::*;
// substrate
use sp_std::prelude::*;

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
		handle.record_cost(BLS_ESTIMATED_COST)?;

		let asig =
			Signature::from_bytes(signature.as_bytes()).map_err(|_| revert("Invalid signature"))?;
		let public_keys: Result<Vec<PublicKey>, _> =
			pubkeys.into_iter().map(|k| PublicKey::from_bytes(k.as_bytes())).collect();
		let Ok(pks) = public_keys else {
            return Err(revert("Invalid pubkeys"));
        };

		let apk = PublicKey::aggregate(pks);
		let msg = hash_to_curve_g2(message.as_bytes()).map_err(|_| revert("Invalid message"))?;
		Ok(apk.verify(&asig, &msg))
	}
}
