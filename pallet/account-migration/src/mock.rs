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

pub use dc_primitives::*;

// polkadot-sdk
use frame_support::derive_impl;
use sp_io::TestExternalities;
use sp_runtime::BuildStorage;

#[sp_version::runtime_version]
pub const VERSION: sp_version::RuntimeVersion = sp_version::RuntimeVersion {
	spec_name: sp_runtime::create_runtime_str!("Darwinia2"),
	impl_name: sp_runtime::create_runtime_str!("DarwiniaOfficialRust"),
	authoring_version: 0,
	spec_version: 6_9_1_0,
	impl_version: 0,
	apis: sp_version::create_apis_vec!([]),
	transaction_version: 0,
	state_version: 0,
};
frame_support::parameter_types! {
	pub const Version: sp_version::RuntimeVersion = VERSION;
}
#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type Block = frame_system::mocking::MockBlock<Self>;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
	type Version = Version;
}

#[derive_impl(pallet_timestamp::config_preludes::TestDefaultConfig as pallet_timestamp::DefaultConfig)]
impl pallet_timestamp::Config for Runtime {
	type MinimumPeriod = ();
	type Moment = Moment;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig as pallet_balances::DefaultConfig)]
impl pallet_balances::Config for Runtime {
	type AccountStore = System;
	type Balance = Balance;
	type ExistentialDeposit = ();
}

#[cfg(feature = "runtime-benchmarks")]
pub enum BenchmarkHelper {}
#[cfg(feature = "runtime-benchmarks")]
impl pallet_assets::BenchmarkHelper<codec::Compact<AssetId>> for BenchmarkHelper {
	fn create_asset_id_parameter(id: u32) -> codec::Compact<AssetId> {
		(id as u64).into()
	}
}
impl pallet_assets::Config for Runtime {
	type ApprovalDeposit = ();
	type AssetAccountDeposit = ();
	type AssetDeposit = ();
	type AssetId = AssetId;
	type AssetIdParameter = codec::Compact<AssetId>;
	type Balance = Balance;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = BenchmarkHelper;
	type CallbackHandle = ();
	type CreateOrigin =
		frame_support::traits::AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
	type Currency = Balances;
	type Extra = ();
	type ForceOrigin = frame_system::EnsureSigned<AccountId>;
	type Freezer = ();
	type MetadataDepositBase = ();
	type MetadataDepositPerByte = ();
	type RemoveItemsLimit = ();
	type RuntimeEvent = RuntimeEvent;
	type StringLimit = frame_support::traits::ConstU32<4>;
	type WeightInfo = ();
}

frame_support::parameter_types! {
	pub UnvestedFundsAllowedWithdrawReasons: frame_support::traits::WithdrawReasons =
		frame_support::traits::WithdrawReasons::except(
			frame_support::traits::WithdrawReasons::TRANSFER | frame_support::traits::WithdrawReasons::RESERVE
		);
}

impl darwinia_deposit::Config for Runtime {
	type DepositMigrator = ();
	type Ring = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Treasury = ();
	type WeightInfo = ();
}

impl crate::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

frame_support::construct_runtime! {
	pub enum Runtime {
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		Assets: pallet_assets,
		Deposit: darwinia_deposit,
		AccountMigration: crate,
	}
}

pub(crate) fn new_test_ext() -> TestExternalities {
	let mut storage = <frame_system::GenesisConfig<Runtime>>::default().build_storage().unwrap();

	pallet_assets::GenesisConfig::<Runtime> {
		assets: vec![(crate::KTON_ID, [0; 20].into(), true, 1)],
		metadata: vec![(crate::KTON_ID, b"KTON".to_vec(), b"KTON".to_vec(), 18)],
		..Default::default()
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	storage.into()
}
