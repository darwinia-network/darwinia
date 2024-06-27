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

frame_support::parameter_types! {
	pub const PreimageBaseDeposit: Balance = darwinia_deposit(2, 64);
	pub const PreimageByteDeposit: Balance = darwinia_deposit(0, 1);
	pub const PreimageHoldReason: RuntimeHoldReason = RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
}

impl pallet_preimage::Config for Runtime {
	type Consideration = frame_support::traits::fungible::HoldConsideration<
		Self::AccountId,
		Balances,
		PreimageHoldReason,
		frame_support::traits::LinearStoragePrice<
			PreimageBaseDeposit,
			PreimageByteDeposit,
			Balance,
		>,
	>;
	type Currency = Balances;
	type ManagerOrigin = RootOr<GeneralAdmin>;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_preimage::WeightInfo<Self>;
}
