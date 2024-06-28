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

// crates.io
use codec::Encode;
// darwinia
use dc_primitives::*;
// polkadot-sdk
use sp_core::crypto::FromEntropy;

/// Helper for pallet-assets benchmarking.
pub enum Assets {}
impl pallet_assets::BenchmarkHelper<codec::Compact<u64>> for Assets {
	fn create_asset_id_parameter(id: u32) -> codec::Compact<u64> {
		u64::from(id).into()
	}
}

pub enum Treasury {}
impl<AssetKind> pallet_treasury::ArgumentsFactory<AssetKind, AccountId> for Treasury
where
	AssetKind: FromEntropy,
{
	fn create_asset_kind(seed: u32) -> AssetKind {
		AssetKind::from_entropy(&mut seed.encode().as_slice()).unwrap()
	}

	fn create_beneficiary(seed: [u8; 32]) -> AccountId {
		<[u8; 20]>::from_entropy(&mut seed.as_slice()).unwrap().into()
	}
}
