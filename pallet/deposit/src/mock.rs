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

pub use crate as darwinia_deposit;
pub use dc_types::{Balance, Moment, UNIT};

// polkadot-sdk
use frame_support::derive_impl;
use sp_io::TestExternalities;
use sp_runtime::BuildStorage;

pub type AssetId = u32;

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type Block = frame_system::mocking::MockBlock<Self>;
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

impl pallet_assets::Config for Runtime {
	type ApprovalDeposit = ();
	type AssetAccountDeposit = ();
	type AssetDeposit = ();
	type AssetId = AssetId;
	type AssetIdParameter = codec::Compact<AssetId>;
	type Balance = Balance;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
	type CallbackHandle = ();
	type CreateOrigin = frame_support::traits::AsEnsureOriginWithArg<
		frame_system::EnsureSignedBy<frame_support::traits::IsInVec<()>, Self::AccountId>,
	>;
	type Currency = Balances;
	type Extra = ();
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type Freezer = ();
	type MetadataDepositBase = ();
	type MetadataDepositPerByte = ();
	type RemoveItemsLimit = ();
	type RuntimeEvent = RuntimeEvent;
	type StringLimit = frame_support::traits::ConstU32<4>;
	type WeightInfo = ();
}

pub enum KtonMinting {}
impl darwinia_deposit::SimpleAsset for KtonMinting {
	type AccountId = <Runtime as frame_system::Config>::AccountId;

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
	type Kton = KtonMinting;
	type MaxDeposits = frame_support::traits::ConstU32<16>;
	type MinLockingAmount = frame_support::traits::ConstU128<UNIT>;
	type Ring = Balances;
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
	}
}

pub(crate) fn efflux(milli_secs: Moment) {
	Timestamp::set_timestamp(Timestamp::now() + milli_secs);
}

pub(crate) fn new_test_ext() -> TestExternalities {
	let mut storage = <frame_system::GenesisConfig<Runtime>>::default().build_storage().unwrap();

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
