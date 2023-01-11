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
use darwinia_common_runtime::gov_origin::ROOT;
use dc_primitives::{AccountId, Balance};
use pangolin_runtime::{Runtime, System};
// parity
use frame_support::traits::GenesisBuild;
use sp_io::TestExternalities;

pub(crate) const KTON_ID: u64 = 1026;

#[derive(Default, Clone)]
pub struct ExtBuilder {
	balances: Vec<(AccountId, Balance)>,
	assets_accounts: Vec<(u64, AccountId, Balance)>,
}

impl ExtBuilder {
	pub fn build(&mut self) -> TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

		pallet_balances::GenesisConfig::<Runtime> { balances: self.balances.clone() }
			.assimilate_storage(&mut t)
			.unwrap();

		pallet_assets::GenesisConfig::<Runtime> {
			assets: vec![(KTON_ID, ROOT, true, 1)],
			metadata: vec![(KTON_ID, b"Test Commitment Token".to_vec(), b"TKTON".to_vec(), 18)],
			accounts: self.assets_accounts.clone(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	pub fn with_balances(&mut self, balances: Vec<(AccountId, Balance)>) -> &mut Self {
		self.balances = balances;
		self
	}

	pub fn with_assets_accounts(&mut self, accounts: Vec<(u64, AccountId, Balance)>) -> &mut Self {
		self.assets_accounts = accounts;
		self
	}
}
