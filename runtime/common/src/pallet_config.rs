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

// darwinia
use dc_primitives::*;
// polkadot-sdk
use sp_core::U256;
use sp_runtime::traits::AccountIdConversion;
use sp_std::prelude::*;

/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: sp_runtime::Perbill = sp_runtime::Perbill::from_percent(5);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
pub const NORMAL_DISPATCH_RATIO: sp_runtime::Perbill = sp_runtime::Perbill::from_percent(75);
const WEIGHT_MILLISECS_PER_BLOCK: u64 = 500;
pub const MAXIMUM_BLOCK_WEIGHT: frame_support::weights::Weight =
	frame_support::weights::Weight::from_parts(
		frame_support::weights::constants::WEIGHT_REF_TIME_PER_MILLIS * WEIGHT_MILLISECS_PER_BLOCK,
		cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64,
	);

#[cfg(not(feature = "runtime-benchmarks"))]
const EXISTENTIAL_DEPOSIT: Balance = 0;
#[cfg(feature = "runtime-benchmarks")]
const EXISTENTIAL_DEPOSIT: Balance = 1;

const BLOCK_GAS_LIMIT: u64 = 20_000_000;

frame_support::parameter_types! {
	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
	pub const MaxBalance: Balance = Balance::max_value();

	// Retry a scheduled item every 10 blocks (1 minute) until the preimage exists.
	pub const NoPreimagePostponement: Option<u32> = Some(10);

	pub const TreasuryPid: frame_support::PalletId = frame_support::PalletId(*b"da/trsry");

	pub const RelayOrigin: cumulus_primitives_core::AggregateMessageOrigin = cumulus_primitives_core::AggregateMessageOrigin::Parent;

	pub const ReservedXcmpWeight: frame_support::weights::Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: frame_support::weights::Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);

	pub RuntimeBlockLength: frame_system::limits::BlockLength =
		frame_system::limits::BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights::builder()
		.base_block(frame_support::weights::constants::BlockExecutionWeight::get())
		.for_class(frame_support::dispatch::DispatchClass::all(), |weights| {
			weights.base_extrinsic = frame_support::weights::constants::ExtrinsicBaseWeight::get();
		})
		.for_class(frame_support::dispatch::DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(frame_support::dispatch::DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();

	pub MaximumSchedulerWeight: frame_support::weights::Weight = sp_runtime::Perbill::from_percent(80)
		* RuntimeBlockWeights::get().max_block;

	pub AssetCreators: Vec<AccountId> = vec![super::gov_origin::ROOT];
	pub TreasuryAccount: AccountId = TreasuryPid::get().into_account_truncating();

	pub MaxProposalWeight: frame_support::weights::Weight = sp_runtime::Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;

	pub MessageQueueServiceWeight: frame_support::weights::Weight = sp_runtime::Perbill::from_percent(35) * RuntimeBlockWeights::get().max_block;

	pub BlockGasLimit: U256 = U256::from(BLOCK_GAS_LIMIT);
	// Restrict the POV size of the Ethereum transactions in the same way as weight limit.
	pub BlockPovSizeLimit: u64 = NORMAL_DISPATCH_RATIO * cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64;
	pub WeightPerGas: frame_support::weights::Weight = frame_support::weights::Weight::from_parts(
		fp_evm::weight_per_gas(BLOCK_GAS_LIMIT, NORMAL_DISPATCH_RATIO, WEIGHT_MILLISECS_PER_BLOCK),
		0
	);
	// FIXME: https://github.com/rust-lang/rust/issues/88581
	pub GasLimitPovSizeRatio: u64 = BLOCK_GAS_LIMIT.saturating_div(BlockPovSizeLimit::get()) + 1;
}
