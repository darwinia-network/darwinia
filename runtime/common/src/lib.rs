// This file is part of Darwinia.
//
// Copyright (C) 2018-2021 Darwinia Network
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

//! Common runtime code for Darwinia and Crab.

#![cfg_attr(not(feature = "std"), no_std)]

/// Implementations of some helper traits passed into runtime modules as associated types.
pub mod impls;
pub use impls::*;

pub use frame_support::weights::constants::{
	BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight,
};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

#[cfg(feature = "std")]
pub use darwinia_staking::StakerStatus;

pub use darwinia_balances::Instance1 as RingInstance;
pub use darwinia_balances::Instance2 as KtonInstance;

// --- crates.io ---
use static_assertions::const_assert;
// --- paritytech ---
use frame_support::{
	parameter_types,
	traits::Currency,
	weights::{constants::WEIGHT_PER_SECOND, DispatchClass, Weight},
};
use frame_system::limits::{BlockLength, BlockWeights};
use pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment};
use sp_runtime::{FixedPointNumber, Perbill, Perquintill};
// --- darwinia-network ---
use darwinia_primitives::BlockNumber;

pub type NegativeImbalance<T> = <darwinia_balances::Pallet<T, RingInstance> as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

/// We assume that an on-initialize consumes 2.5% of the weight on average, hence a single extrinsic
/// will not be allowed to consume more than `AvailableBlockRatio - 2.5%`.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_perthousand(25);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time.
pub const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;
const_assert!(NORMAL_DISPATCH_RATIO.deconstruct() >= AVERAGE_ON_INITIALIZE_RATIO.deconstruct());
parameter_types! {
	pub const BlockHashCountForCrab: BlockNumber = 256;
	pub const BlockHashCountForDarwinia: BlockNumber = 2400;
	/// The portion of the `NORMAL_DISPATCH_RATIO` that we adjust the fees with. Blocks filled less
	/// than this will decrease the weight and more will increase.
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	/// The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
	/// change the fees more rapidly.
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(3, 100_000);
	/// Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
	/// that combined with `AdjustmentVariable`, we can recover from the minimum.
	/// See `multiplier_can_grow_from_zero`.
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
	/// Maximum length of block. Up to 5MB.
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	/// Block weights base values and limits.
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
}

parameter_types! {
	/// A limit for off-chain phragmen unsigned solution submission.
	///
	/// We want to keep it as high as possible, but can't risk having it reject,
	/// so we always subtract the base block execution weight.
	pub OffchainSolutionWeightLimit: Weight = RuntimeBlockWeights::get()
		.get(DispatchClass::Normal)
		.max_extrinsic
		.expect("Normal extrinsics have weight limit configured by default; qed")
		.saturating_sub(BlockExecutionWeight::get());

	/// A limit for off-chain phragmen unsigned solution length.
	///
	/// We allow up to 90% of the block's size to be consumed by the solution.
	pub OffchainSolutionLengthLimit: u32 = Perbill::from_rational(90_u32, 100) *
		*RuntimeBlockLength::get()
		.max
		.get(DispatchClass::Normal);
}

/// Parameterized slow adjusting fee updated based on
/// https://w3f-research.readthedocs.io/en/latest/polkadot/Token%20Economics.html#-2.-slow-adjusting-mechanism
pub type SlowAdjustingFeeUpdate<R> =
	TargetedFeeAdjustment<R, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;

pub fn migrate_treasury() {
	// --- crates.io ---
	use codec::{Decode, Encode};
	// --- paritytech ---
	use frame_support::{migration, StorageHasher, Twox64Concat};
	use sp_std::prelude::*;
	// --- darwinia-network ---
	use darwinia_primitives::*;

	type ProposalIndex = u32;

	const OLD_PREFIX: &[u8] = b"DarwiniaTreasury";
	const NEW_PREFIX: &[u8] = b"Treasury";
	const KTON_TREASURY_PREFIX: &[u8] = b"Instance2Treasury";

	migration::remove_storage_prefix(OLD_PREFIX, b"ProposalCount", &[]);
	log::info!("`ProposalCount` Removed");
	let approvals =
		migration::take_storage_value::<Vec<ProposalIndex>>(OLD_PREFIX, b"Approvals", &[])
			.unwrap_or_default();
	migration::remove_storage_prefix(OLD_PREFIX, b"Approvals", &[]);
	log::info!("`Approvals` Removed");

	#[derive(Encode, Decode)]
	struct OldProposal {
		proposer: AccountId,
		beneficiary: AccountId,
		ring_value: Balance,
		kton_value: Balance,
		ring_bond: Balance,
		kton_bond: Balance,
	}
	#[derive(Encode, Decode)]
	struct Proposal {
		proposer: AccountId,
		value: Balance,
		beneficiary: AccountId,
		bond: Balance,
	}
	let mut ring_proposals_count = 0 as ProposalIndex;
	let mut kton_proposals_count = 0 as ProposalIndex;
	let mut ring_approvals = vec![];
	let mut kton_approvals = vec![];
	for (index, old_proposal) in migration::storage_key_iter::<
		ProposalIndex,
		OldProposal,
		Twox64Concat,
	>(OLD_PREFIX, b"Proposals")
	.drain()
	{
		let hash = Twox64Concat::hash(&index.encode());

		if old_proposal.ring_value != 0 {
			let new_proposal = Proposal {
				proposer: old_proposal.proposer.clone(),
				value: old_proposal.ring_value,
				beneficiary: old_proposal.beneficiary.clone(),
				bond: old_proposal.ring_bond,
			};

			ring_proposals_count += 1;

			migration::put_storage_value(NEW_PREFIX, b"Proposals", &hash, new_proposal);

			if approvals.contains(&index) {
				ring_approvals.push(index);
			}
		}
		if old_proposal.kton_value != 0 {
			let new_proposal = Proposal {
				proposer: old_proposal.proposer,
				value: old_proposal.kton_value,
				beneficiary: old_proposal.beneficiary,
				bond: old_proposal.kton_bond,
			};

			kton_proposals_count += 1;

			migration::put_storage_value(KTON_TREASURY_PREFIX, b"Proposals", &hash, new_proposal);

			if approvals.contains(&index) {
				kton_approvals.push(index);
			}
		}
	}
	if ring_proposals_count != 0 {
		migration::put_storage_value(NEW_PREFIX, b"ProposalCount", &[], ring_proposals_count);
	}
	if kton_proposals_count != 0 {
		migration::put_storage_value(
			KTON_TREASURY_PREFIX,
			b"ProposalCount",
			&[],
			kton_proposals_count,
		);
	}
	migration::remove_storage_prefix(OLD_PREFIX, b"Proposals", &[]);
	log::info!("`Proposals` Migrated");

	if !ring_approvals.is_empty() {
		migration::put_storage_value(NEW_PREFIX, b"Approvals", &[], ring_approvals);
	}
	if !kton_approvals.is_empty() {
		migration::put_storage_value(KTON_TREASURY_PREFIX, b"Approvals", &[], kton_approvals);
	}
	log::info!("`Approvals` Migrated");

	migration::move_storage_from_pallet(b"Tips", OLD_PREFIX, NEW_PREFIX);
	log::info!("`Tips` Migrated");
	migration::move_storage_from_pallet(b"BountyCount", OLD_PREFIX, NEW_PREFIX);
	log::info!("`BountyCount` Migrated");
	migration::move_storage_from_pallet(b"Bounties", OLD_PREFIX, NEW_PREFIX);
	log::info!("`Bounties` Migrated");
	migration::move_storage_from_pallet(b"BountyDescriptions", OLD_PREFIX, NEW_PREFIX);
	log::info!("`BountyDescriptions` Migrated");
	migration::move_storage_from_pallet(b"BountyApprovals", OLD_PREFIX, NEW_PREFIX);
	log::info!("`BountyApprovals` Migrated");
}
