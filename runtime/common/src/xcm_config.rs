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

// core
use core::marker::PhantomData;
// crates.io
use codec::Encode;
// darwinia
use dc_primitives::GWEI;
// polkadot-sdk
use frame_support::{
	pallet_prelude::*,
	traits::{ConstU128, Contains},
};
use sp_io::hashing::blake2_256;
use sp_std::prelude::*;
use xcm::latest::prelude::*;
use xcm_executor::traits::ConvertLocation;

/// Base balance required for the XCM unit weight.
pub type XcmBaseWeightFee = ConstU128<GWEI>;

/// Struct that converts a given Location into a 20 bytes account id by hashing
/// with blake2_256 and taking the first 20 bytes
pub struct Account20Hash<AccountId>(PhantomData<AccountId>);
impl<AccountId: From<[u8; 20]> + Into<[u8; 20]> + Clone> ConvertLocation<AccountId>
	for Account20Hash<AccountId>
{
	fn convert_location(location: &Location) -> Option<AccountId> {
		let hash: [u8; 32] = ("multiloc", location).using_encoded(blake2_256);
		let mut account_id = [0u8; 20];

		account_id.copy_from_slice(&hash[0..20]);

		Some(account_id.into())
	}
}

pub struct ParentOrParentsPlurality;
impl Contains<Location> for ParentOrParentsPlurality {
	fn contains(location: &Location) -> bool {
		matches!(
			location.unpack(),
			(1, [])
				| (1, [Plurality { id: BodyId::Administration, .. }])
				| (1, [Plurality { id: BodyId::Executive, .. }])
				| (1, [Plurality { id: BodyId::Technical, .. }])
		)
	}
}

/// Filter to check if a given location is the parent Relay Chain or a sibling parachain.
///
/// This type should only be used within the context of a parachain, since it does not verify that
/// the parent is indeed a Relay Chain.
pub struct ParentRelayOrSiblingParachains;
impl Contains<Location> for ParentRelayOrSiblingParachains {
	fn contains(location: &Location) -> bool {
		matches!(location.unpack(), (1, []) | (1, [Parachain(_)]))
	}
}

// TODO: move to other place.
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub struct AssetRegistrarMetadata {
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
	pub decimals: u8,
	pub is_frozen: bool,
}
