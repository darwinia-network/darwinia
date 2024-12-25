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

impl pallet_xcm_weight_trader::Config for Runtime {
	type AccountIdToLocation = xcm_primitives::AccountIdToLocation<AccountId>;
	type AddSupportedAssetOrigin = Root;
	type AssetLocationFilter = frame_support::traits::Everything;
	type AssetTransactor = AssetTransactors;
	type Balance = Balance;
	type EditSupportedAssetOrigin = Root;
	type NativeLocation = SelfReserve;
	#[cfg(feature = "runtime-benchmarks")]
	type NotFilteredLocation = RelayLocation;
	type PauseSupportedAssetOrigin = Root;
	type RemoveSupportedAssetOrigin = Root;
	type ResumeSupportedAssetOrigin = Root;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_xcm_weight_trader::WeightInfo<Runtime>;
	type WeightToFee = <Runtime as pallet_transaction_payment::Config>::WeightToFee;
	type XcmFeesAccount = XcmFeesAccount;
}
