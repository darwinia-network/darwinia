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
#[allow(unused_imports)]
use crate::*;
// substrate
#[allow(unused_imports)]
use frame_support::{log, migration, storage::unhashed};

pub struct CustomOnRuntimeUpgrade;
impl frame_support::traits::OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
		Ok(())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		migrate()
	}
}

fn migrate() -> frame_support::weights::Weight {
	// substrate
	use frame_support::PalletId;
	use pallet_balances::WeightInfo;
	use sp_runtime::traits::AccountIdConversion;

	let staking: AccountId = PalletId(*b"da/staki").into_account_truncating();
	let deprecate_staking: AccountId = PalletId(*b"dar/stak").into_account_truncating();

	let _ = Balances::transfer_all(RuntimeOrigin::signed(deprecate_staking), staking, false);

	// frame_support::weights::Weight::zero()
	// RuntimeBlockWeights::get().max_block
	// <Runtime as frame_system::Config>::DbWeight::get().reads_writes(3, 3)
	<Runtime as pallet_balances::Config>::WeightInfo::transfer_all()
}
