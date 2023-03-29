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

//! Autogenerated weights for darwinia_ecdsa_authority
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-03-22, STEPS: `2`, REPEAT: `1`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `inv.cafe`, CPU: `13th Gen Intel(R) Core(TM) i9-13900K`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("pangolin-local"), DB CACHE: 1024

// Executed Command:
// target/release/darwinia
// benchmark
// pallet
// --header
// .maintain/license-header
// --template
// .maintain/pallet-weight-template.hbs
// --execution
// wasm
// --heap-pages
// 4096
// --chain
// pangolin-local
// --output
// pallet/ecdsa-authority/src/weights.rs
// --extrinsic
// *
// --pallet
// darwinia-ecdsa-authority

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(missing_docs)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for darwinia_ecdsa_authority.
pub trait WeightInfo {
	fn on_initialize() -> Weight;
	fn add_authority() -> Weight;
	fn remove_authority() -> Weight;
	fn swap_authority() -> Weight;
	fn submit_authorities_change_signature() -> Weight;
	fn submit_new_message_root_signature() -> Weight;
}

/// Weights for darwinia_ecdsa_authority using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: EcdsaAuthority AuthoritiesChangeToSign (r:1 w:0)
	/// Proof Skipped: EcdsaAuthority AuthoritiesChangeToSign (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority MessageRootToSign (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority MessageRootToSign (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority Nonce (r:1 w:0)
	/// Proof Skipped: EcdsaAuthority Nonce (max_values: Some(1), max_size: None, mode: Measured)
	fn on_initialize() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `894`
		//  Estimated: `4167`
		// Minimum execution time: 23_399 nanoseconds.
		Weight::from_ref_time(23_399_000)
			.saturating_add(Weight::from_proof_size(4167))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: EcdsaAuthority AuthoritiesChangeToSign (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority AuthoritiesChangeToSign (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority NextAuthorities (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority NextAuthorities (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority Nonce (r:1 w:0)
	/// Proof Skipped: EcdsaAuthority Nonce (max_values: Some(1), max_size: None, mode: Measured)
	fn add_authority() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1071`
		//  Estimated: `4698`
		// Minimum execution time: 34_089 nanoseconds.
		Weight::from_ref_time(34_089_000)
			.saturating_add(Weight::from_proof_size(4698))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: EcdsaAuthority AuthoritiesChangeToSign (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority AuthoritiesChangeToSign (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority NextAuthorities (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority NextAuthorities (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority Nonce (r:1 w:0)
	/// Proof Skipped: EcdsaAuthority Nonce (max_values: Some(1), max_size: None, mode: Measured)
	fn remove_authority() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1182`
		//  Estimated: `5031`
		// Minimum execution time: 40_182 nanoseconds.
		Weight::from_ref_time(40_182_000)
			.saturating_add(Weight::from_proof_size(5031))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: EcdsaAuthority AuthoritiesChangeToSign (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority AuthoritiesChangeToSign (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority NextAuthorities (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority NextAuthorities (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority Nonce (r:1 w:0)
	/// Proof Skipped: EcdsaAuthority Nonce (max_values: Some(1), max_size: None, mode: Measured)
	fn swap_authority() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1202`
		//  Estimated: `5091`
		// Minimum execution time: 25_077 nanoseconds.
		Weight::from_ref_time(25_077_000)
			.saturating_add(Weight::from_proof_size(5091))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: EcdsaAuthority Authorities (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority Authorities (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority AuthoritiesChangeToSign (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority AuthoritiesChangeToSign (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority NextAuthorities (r:1 w:0)
	/// Proof Skipped: EcdsaAuthority NextAuthorities (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority Nonce (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority Nonce (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority MessageRootToSign (r:0 w:1)
	/// Proof Skipped: EcdsaAuthority MessageRootToSign (max_values: Some(1), max_size: None, mode: Measured)
	fn submit_authorities_change_signature() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1299`
		//  Estimated: `8475`
		// Minimum execution time: 33_450 nanoseconds.
		Weight::from_ref_time(33_450_000)
			.saturating_add(Weight::from_proof_size(8475))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	/// Storage: EcdsaAuthority Authorities (r:1 w:0)
	/// Proof Skipped: EcdsaAuthority Authorities (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority MessageRootToSign (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority MessageRootToSign (max_values: Some(1), max_size: None, mode: Measured)
	fn submit_new_message_root_signature() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1260`
		//  Estimated: `3510`
		// Minimum execution time: 27_202 nanoseconds.
		Weight::from_ref_time(27_202_000)
			.saturating_add(Weight::from_proof_size(3510))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: EcdsaAuthority AuthoritiesChangeToSign (r:1 w:0)
	/// Proof Skipped: EcdsaAuthority AuthoritiesChangeToSign (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority MessageRootToSign (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority MessageRootToSign (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority Nonce (r:1 w:0)
	/// Proof Skipped: EcdsaAuthority Nonce (max_values: Some(1), max_size: None, mode: Measured)
	fn on_initialize() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `894`
		//  Estimated: `4167`
		// Minimum execution time: 23_399 nanoseconds.
		Weight::from_ref_time(23_399_000)
			.saturating_add(Weight::from_proof_size(4167))
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: EcdsaAuthority AuthoritiesChangeToSign (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority AuthoritiesChangeToSign (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority NextAuthorities (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority NextAuthorities (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority Nonce (r:1 w:0)
	/// Proof Skipped: EcdsaAuthority Nonce (max_values: Some(1), max_size: None, mode: Measured)
	fn add_authority() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1071`
		//  Estimated: `4698`
		// Minimum execution time: 34_089 nanoseconds.
		Weight::from_ref_time(34_089_000)
			.saturating_add(Weight::from_proof_size(4698))
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: EcdsaAuthority AuthoritiesChangeToSign (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority AuthoritiesChangeToSign (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority NextAuthorities (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority NextAuthorities (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority Nonce (r:1 w:0)
	/// Proof Skipped: EcdsaAuthority Nonce (max_values: Some(1), max_size: None, mode: Measured)
	fn remove_authority() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1182`
		//  Estimated: `5031`
		// Minimum execution time: 40_182 nanoseconds.
		Weight::from_ref_time(40_182_000)
			.saturating_add(Weight::from_proof_size(5031))
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: EcdsaAuthority AuthoritiesChangeToSign (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority AuthoritiesChangeToSign (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority NextAuthorities (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority NextAuthorities (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority Nonce (r:1 w:0)
	/// Proof Skipped: EcdsaAuthority Nonce (max_values: Some(1), max_size: None, mode: Measured)
	fn swap_authority() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1202`
		//  Estimated: `5091`
		// Minimum execution time: 25_077 nanoseconds.
		Weight::from_ref_time(25_077_000)
			.saturating_add(Weight::from_proof_size(5091))
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: EcdsaAuthority Authorities (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority Authorities (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority AuthoritiesChangeToSign (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority AuthoritiesChangeToSign (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority NextAuthorities (r:1 w:0)
	/// Proof Skipped: EcdsaAuthority NextAuthorities (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority Nonce (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority Nonce (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority MessageRootToSign (r:0 w:1)
	/// Proof Skipped: EcdsaAuthority MessageRootToSign (max_values: Some(1), max_size: None, mode: Measured)
	fn submit_authorities_change_signature() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1299`
		//  Estimated: `8475`
		// Minimum execution time: 33_450 nanoseconds.
		Weight::from_ref_time(33_450_000)
			.saturating_add(Weight::from_proof_size(8475))
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
	/// Storage: EcdsaAuthority Authorities (r:1 w:0)
	/// Proof Skipped: EcdsaAuthority Authorities (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: EcdsaAuthority MessageRootToSign (r:1 w:1)
	/// Proof Skipped: EcdsaAuthority MessageRootToSign (max_values: Some(1), max_size: None, mode: Measured)
	fn submit_new_message_root_signature() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1260`
		//  Estimated: `3510`
		// Minimum execution time: 27_202 nanoseconds.
		Weight::from_ref_time(27_202_000)
			.saturating_add(Weight::from_proof_size(3510))
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}
