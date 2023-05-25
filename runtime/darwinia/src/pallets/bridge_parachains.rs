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

pub use pallet_bridge_parachains::Instance1 as WithKusamaParachainsInstance;

// darwinia
use crate::*;

frame_support::parameter_types! {
	pub const ParasPalletName: &'static str = bp_polkadot_core::parachains::PARAS_PALLET_NAME;
}

impl pallet_bridge_parachains::Config<WithKusamaParachainsInstance> for Runtime {
	type BridgesGrandpaPalletInstance = WithKusamaGrandpa;
	type HeadsToKeep = KusamaHeadersToKeep;
	type MaxParaHeadSize = ConstU32<1024>;
	type ParasPalletName = ParasPalletName;
	type RuntimeEvent = RuntimeEvent;
	type TrackedParachains = frame_support::traits::Everything;
	type WeightInfo = weights::pallet_bridge_parachains::WeightInfo<Self>;
}
