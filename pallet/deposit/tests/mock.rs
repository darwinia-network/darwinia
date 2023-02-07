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

// darwinia
use dc_types::{AssetId, Balance, Moment, UNIT};
// substrate
use frame_support::traits::GenesisBuild;
use sp_io::TestExternalities;

impl frame_system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = u32;
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockHashCount = ();
	type BlockLength = ();
	type BlockNumber = u64;
	type BlockWeights = ();
	type DbWeight = ();
	type Hash = sp_core::H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type Header = sp_runtime::testing::Header;
	type Index = u64;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SS58Prefix = ();
	type SystemWeightInfo = ();
	type Version = ();
}

impl pallet_timestamp::Config for Runtime {
	type MinimumPeriod = ();
	type Moment = Moment;
	type OnTimestampSet = ();
	type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
	type AccountStore = System;
	type Balance = Balance;
	type DustRemoval = ();
	type ExistentialDeposit = frame_support::traits::ConstU128<0>;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

impl pallet_assets::Config for Runtime {
	type ApprovalDeposit = ();
	type AssetAccountDeposit = ();
	type AssetDeposit = ();
	type AssetId = AssetId;
	type AssetIdParameter = codec::Compact<AssetId>;
	type Balance = Balance;
	type CreateOrigin = frame_support::traits::AsEnsureOriginWithArg<
		frame_system::EnsureSignedBy<frame_support::traits::IsInVec<()>, u32>,
	>;
	type Currency = Balances;
	type Extra = ();
	type ForceOrigin = frame_system::EnsureRoot<u32>;
	type Freezer = ();
	type MetadataDepositBase = ();
	type MetadataDepositPerByte = ();
	type RemoveItemsLimit = ();
	type RuntimeEvent = RuntimeEvent;
	type StringLimit = frame_support::traits::ConstU32<4>;
	type WeightInfo = ();
}

pub enum KtonAsset {}
impl darwinia_deposit::SimpleAsset for KtonAsset {
	type AccountId = u32;

	fn mint(beneficiary: &Self::AccountId, amount: Balance) -> sp_runtime::DispatchResult {
		Assets::mint(RuntimeOrigin::signed(0), 0.into(), *beneficiary, amount)
	}

	fn burn(who: &Self::AccountId, amount: Balance) -> sp_runtime::DispatchResult {
		if Assets::balance(0, who) < amount {
			Err(<pallet_assets::Error<Runtime>>::BalanceLow)?;
		}

		Assets::burn(RuntimeOrigin::signed(0), 0.into(), *who, amount)
	}
}
impl darwinia_deposit::Config for Runtime {
	type Kton = KtonAsset;
	type MaxDeposits = frame_support::traits::ConstU32<16>;
	type MinLockingAmount = frame_support::traits::ConstU128<UNIT>;
	type Ring = Balances;
	type RuntimeEvent = RuntimeEvent;
	type UnixTime = Timestamp;
}

frame_support::construct_runtime!(
	pub enum Runtime where
		Block = frame_system::mocking::MockBlock<Runtime>,
		NodeBlock = frame_system::mocking::MockBlock<Runtime>,
		UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>,
	{
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		Assets: pallet_assets,
		Deposit: darwinia_deposit,
	}
);

pub fn efflux(milli_secs: Moment) {
	Timestamp::set_timestamp(Timestamp::now() + milli_secs);
}

pub fn new_test_ext() -> TestExternalities {
	let mut storage = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: (1..=2).map(|i| (i, (i as Balance) * 1_000 * UNIT)).collect(),
	}
	.assimilate_storage(&mut storage)
	.unwrap();
	pallet_assets::GenesisConfig::<Runtime> {
		assets: vec![(0, 0, true, 1)],
		metadata: vec![(0, b"KTON".to_vec(), b"KTON".to_vec(), 18)],
		..Default::default()
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	storage.into()
}
