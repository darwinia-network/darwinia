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
use crate::*;

pub enum PKtonMinting {}
impl darwinia_deposit::SimpleAsset for PKtonMinting {
	type AccountId = AccountId;

	fn mint(beneficiary: &Self::AccountId, amount: Balance) -> sp_runtime::DispatchResult {
		Assets::mint(
			RuntimeOrigin::signed(ROOT),
			(AssetIds::PKton as AssetId).into(),
			*beneficiary,
			amount,
		)
	}

	fn burn(who: &Self::AccountId, amount: Balance) -> sp_runtime::DispatchResult {
		let asset_id = AssetIds::PKton as _;

		if Assets::balance(asset_id, who) < amount {
			Err(<pallet_assets::Error<Runtime>>::BalanceLow)?;
		}

		Assets::burn(RuntimeOrigin::signed(ROOT), asset_id.into(), *who, amount)
	}
}

impl darwinia_deposit::Config for Runtime {
	type Kton = PKtonMinting;
	type MaxDeposits = frame_support::traits::ConstU32<16>;
	type MinLockingAmount = frame_support::traits::ConstU128<UNIT>;
	type Ring = Balances;
	type RuntimeEvent = RuntimeEvent;
	type UnixTime = Timestamp;
}
