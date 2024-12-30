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

frame_support::parameter_types! {
	pub const Version: sp_version::RuntimeVersion = VERSION;
}

#[derive_impl(frame_system::config_preludes::ParaChainDefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type BaseCallFilter = TxPause;
	type Block = Block;
	type BlockLength = pallet_config::RuntimeBlockLength;
	type BlockWeights = pallet_config::RuntimeBlockWeights;
	type DbWeight = frame_support::weights::constants::RocksDbWeight;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
	type MaxConsumers = ConstU32<16>;
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type SS58Prefix = ConstU16<42>;
	// type SystemWeightInfo = weights::frame_system::WeightInfo<Self>;
	type SystemWeightInfo = ();
	type Version = Version;
}
