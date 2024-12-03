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
use dc_types::{Balance, Moment, UNIT};
// polkadot-sdk
use frame_support::derive_impl;
use sp_io::TestExternalities;
use sp_runtime::BuildStorage;

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

impl crate::Config for Runtime {
	type DepositMigrator = ();
	type Ring = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Treasury = ();
	type WeightInfo = ();
}

frame_support::construct_runtime! {
	pub enum Runtime {
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		Deposit: crate,
	}
}

pub fn new_test_ext() -> TestExternalities {
	let mut storage = <frame_system::GenesisConfig<Runtime>>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: (1..=2).map(|i| (i, (i as Balance) * 1_000 * UNIT)).collect(),
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	storage.into()
}

pub fn events() -> Vec<Event<Runtime>> {
	System::read_events_for_pallet()
}
