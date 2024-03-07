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

//! Primitives of the dp-ecdsa-authority.

pub use sp_core::{ecdsa::Signature, H160 as Address, H256 as Hash};

// core
use core::fmt::Debug;
// crates.io
use codec::{Decode, Encode};
use scale_info::TypeInfo;
// darwinia
use dc_primitives::AccountId;
// substrate
use frame_support::{BoundedVec, EqNoBound, PartialEqNoBound, RuntimeDebugNoBound};
use sp_core::Get;
use sp_io::{crypto, hashing};
use sp_runtime::RuntimeDebug;

// address(0x1)
pub(crate) const AUTHORITY_SENTINEL: [u8; 20] =
	[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
// keccak256("ChangeRelayer(bytes4 sig,bytes params,uint256 nonce)");
// 0x30a82982a8d5050d1c83bbea574aea301a4d317840a8c4734a308ffaa6a63bc8
pub(crate) const RELAY_TYPE_HASH: [u8; 32] = [
	48, 168, 41, 130, 168, 213, 5, 13, 28, 131, 187, 234, 87, 74, 234, 48, 26, 77, 49, 120, 64,
	168, 196, 115, 74, 48, 143, 250, 166, 166, 59, 200,
];
// keccak256("Commitment(uint32 block_number,bytes32 message_root,uint256 nonce)");
// 0xaca824a0c4edb3b2c17f33fea9cb21b33c7ee16c8e634c36b3bf851c9de7a223
pub(crate) const COMMIT_TYPE_HASH: [u8; 32] = [
	172, 168, 36, 160, 196, 237, 179, 178, 193, 127, 51, 254, 169, 203, 33, 179, 60, 126, 225, 108,
	142, 99, 76, 54, 179, 191, 133, 28, 157, 231, 162, 35,
];

pub(crate) enum Sign {}
impl Sign {
	fn hash(data: &[u8]) -> [u8; 32] {
		hashing::keccak_256(data)
	}

	pub fn domain_separator(chain_id: u64, spec_name: &[u8]) -> [u8; 32] {
		Self::hash(&[&chain_id.to_be_bytes(), spec_name, b"::ecdsa-authority"].concat())
	}

	// \x19\x01 + keccack256(ChainIDSpecName::ecdsa-authority) + struct_hash
	pub(crate) fn signable_message(chain_id: u64, spec_name: &[u8], data: &[u8]) -> Hash {
		Hash(Self::hash(
			&[
				b"\x19\x01".as_slice(),
				&Self::domain_separator(chain_id, spec_name),
				&Self::hash(data),
			]
			.concat(),
		))
	}

	pub(crate) fn verify_signature(
		signature: &[u8; 65],
		message: &[u8; 32],
		address: &[u8],
	) -> bool {
		if let Ok(public_key) = crypto::secp256k1_ecdsa_recover(signature, message) {
			&Self::hash(&public_key)[12..] == address
		} else {
			false
		}
	}
}

/// Operation types of authority changing.
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum Operation<A> {
	/// Add a new member.
	AddMember {
		/// The new member's account id.
		new: A,
	},
	/// Remove a member.
	///
	/// The previous member is required, because the authorities is a linked map stored on the
	/// Ethereum side.
	RemoveMember {
		/// The old member's previous member's account id.
		pre: A,
		/// The old member's account id.
		old: A,
	},
	/// Swap `old` member with `new` member.
	///
	/// The previous member is required, because the authorities is a linked map stored on the
	/// Ethereum side.
	SwapMembers {
		/// The old member's previous member's account id.
		pre: A,
		/// The old member's account id.
		old: A,
		/// The new member's account id.
		new: A,
	},
}
impl<A> Operation<A> {
	pub(crate) fn id(&self) -> [u8; 4] {
		match self {
			// bytes4(keccak256("add_relayer(address,uint256)"))
			// 0xb7aafe32
			Self::AddMember { .. } => [183, 170, 254, 50],
			// bytes4(keccak256("remove_relayer(address,address,uint256)"))
			// 0x8621d1fa
			Self::RemoveMember { .. } => [134, 33, 209, 250],
			// bytes4(keccak256("swap_relayer(address,address,address)"))
			// 0xcb76085b
			Self::SwapMembers { .. } => [203, 118, 8, 91],
		}
	}
}

/// The dp-ecdsa-authority commitment.
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Commitment<Bn> {
	pub(crate) block_number: Bn,
	pub(crate) message_root: Hash,
	pub(crate) nonce: u32,
}

/// The signing state of an authority change request.
///
/// The struct holds the necessary information to verify that the requested change in authority is
/// authentic and contains enough valid signatures to execute the requested operation.
#[derive(PartialEqNoBound, EqNoBound, Decode, Encode, RuntimeDebugNoBound, TypeInfo)]
#[scale_info(skip_type_params(MaxAuthorities))]
pub struct AuthoritiesChangeSigned<MaxAuthorities>
where
	MaxAuthorities: Get<u32>,
{
	pub(crate) operation: Operation<AccountId>,
	pub(crate) threshold: Option<u32>,
	pub(crate) message: Hash,
	pub(crate) signatures: BoundedVec<(AccountId, Signature), MaxAuthorities>,
}

/// The signing state of a new message root.
///
/// The struct holds the necessary information to verify that the message is authorized by the
/// on-chain authorities.
#[derive(PartialEqNoBound, EqNoBound, Decode, Encode, RuntimeDebugNoBound, TypeInfo)]
#[scale_info(skip_type_params(MaxAuthorities))]
pub struct MessageRootSigned<Bn, MaxAuthorities>
where
	Bn: Debug + PartialEq,
	MaxAuthorities: Get<u32>,
{
	pub(crate) commitment: Commitment<Bn>,
	pub(crate) message: Hash,
	pub(crate) signatures: BoundedVec<(AccountId, Signature), MaxAuthorities>,
	pub(crate) authorized: bool,
}

#[test]
fn signable_message() {
	assert_eq!(
		array_bytes::bytes2hex("0x", Sign::domain_separator(46, b"Darwinia").as_ref()),
		"0xc494742e979bd6ab2dca4950fddd8809e1502ab8ef7b8d749364ec32cb6e1b3e"
	);
	assert_eq!(
		array_bytes::bytes2hex("0x", Sign::domain_separator(43, b"Pangolin2").as_ref()),
		"0xe97c73e46305f3bca2279f002665725cd29e465c6624e83a135f7b2e6b1a8134"
	);
	assert_eq!(
		array_bytes::bytes2hex("0x", Sign::domain_separator(46, b"Darwinia2").as_ref()),
		"0x8cd92680eceac15cdb15004d4a8e42e7ac6e0b506a9664b4aa1bbe35df32c218"
	);

	assert_eq!(
		array_bytes::bytes2hex("0x", Sign::signable_message(46, b"Darwinia", &[0; 32])),
		"0xe52c7ebc7e478b623a16cc38469eca4aa1255bed6cd2599e529080d27ecaed32"
	);
	assert_eq!(
		array_bytes::bytes2hex("0x", Sign::signable_message(45, b"Pangoro", &array_bytes::hex2bytes_unchecked("0x30a82982a8d5050d1c83bbea574aea301a4d317840a8c4734a308ffaa6a63bc8cb76085b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000100000000000000000000000068898db1012808808c903f390909c52d9f7067490000000000000000000000004cdc1dbbd754ea539f1ffaea91f1b6c4b8dd14bd"))),
		"0x9688cfb794c97094994409ed1c7c7caad076ae53ec13af8a370b55654f7bcb36"
	);

	let operation = Operation::SwapMembers {
		pre: AUTHORITY_SENTINEL,
		old: AUTHORITY_SENTINEL,
		new: AUTHORITY_SENTINEL,
	};
	let encoded = ethabi::encode(&[
		ethabi::Token::FixedBytes(RELAY_TYPE_HASH.into()),
		ethabi::Token::FixedBytes(operation.id().into()),
		ethabi::Token::Bytes(ethabi::encode(&[
			ethabi::Token::Address(AUTHORITY_SENTINEL.into()),
			ethabi::Token::Address(AUTHORITY_SENTINEL.into()),
			ethabi::Token::Address(AUTHORITY_SENTINEL.into()),
		])),
		ethabi::Token::Uint(0.into()),
	]);
	assert_eq!(
		array_bytes::bytes2hex("0x", Sign::signable_message(45, b"Pangoro", &encoded)),
		"0x2ca922116daa8c7fec2b58362b58764e83f3b24e906fb50f58e3e1a2e208fb77"
	);
}
