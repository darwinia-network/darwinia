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

/// List of the assets existed in this runtime.
pub enum AssetIds {
	Kton = 1026,
}

impl pallet_assets::Config for Runtime {
	type ApprovalDeposit = ();
	#[cfg(feature = "runtime-benchmarks")]
	type AssetAccountDeposit = ConstU128<1>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type AssetAccountDeposit = ();
	type AssetDeposit = ();
	type AssetId = AssetId;
	type AssetIdParameter = codec::Compact<AssetId>;
	type Balance = Balance;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = benchmark_helper::Assets;
	type CallbackHandle = ();
	type CreateOrigin = frame_support::traits::AsEnsureOriginWithArg<
		frame_system::EnsureSignedBy<
			frame_support::traits::IsInVec<pallet_config::AssetCreators>,
			Self::AccountId,
		>,
	>;
	type Currency = Balances;
	type Extra = ();
	type ForceOrigin = RootOr<GeneralAdmin>;
	type Freezer = ();
	type MetadataDepositBase = ();
	type MetadataDepositPerByte = ();
	type RemoveItemsLimit = ConstU32<1000>;
	type RuntimeEvent = RuntimeEvent;
	type StringLimit = ConstU32<50>;
	// type WeightInfo = weights::pallet_assets::WeightInfo<Self>;
	type WeightInfo = ();
}
