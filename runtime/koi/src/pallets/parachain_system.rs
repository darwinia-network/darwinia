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

impl cumulus_pallet_parachain_system::Config for Runtime {
	type CheckAssociatedRelayNumber =
		cumulus_pallet_parachain_system::RelayNumberMonotonicallyIncreases;
	type ConsensusHook = ConsensusHook;
	type DmpQueue =
		frame_support::traits::EnqueueWithOrigin<MessageQueue, crate::pallet_config::RelayOrigin>;
	type OnSystemEvent = ();
	type OutboundXcmpMessageSource = XcmpQueue;
	type ReservedDmpWeight = crate::pallet_config::ReservedDmpWeight;
	type ReservedXcmpWeight = crate::pallet_config::ReservedXcmpWeight;
	type RuntimeEvent = RuntimeEvent;
	type SelfParaId = parachain_info::Pallet<Self>;
	type WeightInfo = weights::cumulus_pallet_parachain_system::WeightInfo<Self>;
	type XcmpMessageHandler = XcmpQueue;
}
