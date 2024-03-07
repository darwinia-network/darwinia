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

pub enum KtonMinting {}
impl dp_deposit::SimpleAsset for KtonMinting {
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

impl dp_deposit::Config for Runtime {
	type Kton = KtonMinting;
	type MaxDeposits = ConstU32<512>;
	type MinLockingAmount = ConstU128<UNIT>;
	type Ring = Balances;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::dp_deposit::WeightInfo<Self>;
}
