// This file is part of Darwinia.
//
// Copyright (C) 2018-2022 Darwinia Network
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

/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// We allow for 0.5 of a second of compute with a 12 second average block time.
pub const MAXIMUM_BLOCK_WEIGHT: Weight =
	frame_support::weights::constants::WEIGHT_PER_SECOND.saturating_div(2);

frame_support::parameter_types! {
	pub const Version: sp_version::RuntimeVersion = VERSION;
	pub RuntimeBlockLength: frame_system::limits::BlockLength =
		frame_system::limits::BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights::builder()
		.base_block(weights::BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = weights::ExtrinsicBaseWeight::get();
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

impl frame_system::Config for Runtime {
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = frame_support::traits::Everything;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = ConstU32<2400>;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = weights::RocksDbWeight;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = Hashing;
	/// The header type.
	type Header = generic::Header<BlockNumber, Self::Hashing>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = sp_runtime::traits::IdentityLookup<AccountId>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// The action to take on a Runtime Upgrade
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	/// Converts a module to an index of this module in the runtime.
	type PalletInfo = PalletInfo;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = ConstU16<18>;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = weights::frame_system::WeightInfo<Runtime>;
	/// Runtime version.
	type Version = Version;
}
