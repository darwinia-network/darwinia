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
use crate::*;
// polkadot-sdk
use frame_support::derive_impl;

/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
pub const AVERAGE_ON_INITIALIZE_RATIO: sp_runtime::Perbill = sp_runtime::Perbill::from_percent(5);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
pub const NORMAL_DISPATCH_RATIO: sp_runtime::Perbill = sp_runtime::Perbill::from_percent(75);

/// We allow for 0.5 of a second of compute with a 12 second average block time.
pub const WEIGHT_MILLISECS_PER_BLOCK: u64 = 500;
pub const MAXIMUM_BLOCK_WEIGHT: frame_support::weights::Weight =
	frame_support::weights::Weight::from_parts(
		frame_support::weights::constants::WEIGHT_REF_TIME_PER_MILLIS * WEIGHT_MILLISECS_PER_BLOCK,
		cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64,
	);

frame_support::parameter_types! {
	pub const Version: sp_version::RuntimeVersion = VERSION;
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
}

#[derive_impl(frame_system::config_preludes::ParaChainDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type BaseCallFilter = TxPause;
	type Block = Block;
	type BlockLength = RuntimeBlockLength;
	type BlockWeights = RuntimeBlockWeights;
	type DbWeight = frame_support::weights::constants::RocksDbWeight;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
	type MaxConsumers = ConstU32<16>;
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type SS58Prefix = ConstU16<42>;
	// type SystemWeightInfo = weights::frame_system::WeightInfo<Self>;
	type SystemWeightInfo = ();
	type Version = Version;
}
