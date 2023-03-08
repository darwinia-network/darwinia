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
// moonbeam
use precompile_utils::prelude::*;
// substrate
use sp_std::prelude::*;

pub struct BLS12381<T>(PhantomData<T>);

#[precompile_utils::precompile]
impl<Runtime: pallet_evm::Config> BLS12381<Runtime> {
	#[precompile::public("fast_aggregate_verify(bytes[],bytes,bytes)")]
	#[precompile::view]
	fn state_storage_at(
		_handle: &mut impl PrecompileHandle,
		_pubkeys: Vec<UnboundedBytes>,
		_message: UnboundedBytes,
		_signature: UnboundedBytes,
	) -> EvmResult<bool> {
		return Err(revert("Unavailable now"));
	}
}
