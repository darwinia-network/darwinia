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

//! Darwinia chain core primitives.
//!
//! # Opaque types.
//! These are used by the CLI to instantiate machinery that don't need to know
//! the specifics of the runtime. They can then be made to be agnostic over specific formats
//! of data like extrinsics, allowing for them to continue syncing the network through upgrades
//! to even the core data structures.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

pub use dc_types::*;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = fp_account::EthereumSignature;

/// Some way of identifying an account on the chain.
/// We intentionally make it equivalent to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as sp_runtime::traits::Verify>::Signer as sp_runtime::traits::IdentifyAccount>::AccountId;

/// Index of a transaction in the chain.
/// or
/// nonce of an account in the chain.
pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An index to a block.
pub type BlockNumber = u32;

/// The address format for describing accounts.
pub type Address = AccountId;

/// Hashing type
pub type Hashing = sp_runtime::traits::BlakeTwo256;

/// Block header type.
pub type Header = sp_runtime::generic::Header<BlockNumber, Hashing>;

/// Block type.
pub type Block = sp_runtime::generic::Block<Header, sp_runtime::OpaqueExtrinsic>;

/// Maximum number of blocks simultaneously accepted by the Runtime, not yet included
/// into the relay chain.
pub const UNINCLUDED_SEGMENT_CAPACITY: u32 = 3;
/// How many parachain blocks are processed by the relay chain per parent. Limits the
/// number of blocks authored per slot.
pub const BLOCK_PROCESSING_VELOCITY: u32 = 1;
/// Relay chain slot duration, in milliseconds.
pub const RELAY_CHAIN_SLOT_DURATION_MILLIS: u32 = 6_000;

/// Slot duration.
pub const SLOT_DURATION: u64 = 6_000;

// Time is measured by number of blocks.
/// 10 blocks.
pub const MINUTES: BlockNumber = 60_000 / (SLOT_DURATION as BlockNumber);
/// 600 blocks.
pub const HOURS: BlockNumber = MINUTES * 60;
/// 14,400 blocks.
pub const DAYS: BlockNumber = HOURS * 24;
