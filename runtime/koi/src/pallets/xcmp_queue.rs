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

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type ChannelInfo = ParachainSystem;
	type ControllerOrigin = RootOr<GeneralAdmin>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type MaxInboundSuspended = sp_core::ConstU32<1_000>;
	type PriceForSiblingDelivery = polkadot_runtime_common::xcm_sender::NoPriceForMessageDelivery<
		cumulus_primitives_core::ParaId,
	>;
	type RuntimeEvent = RuntimeEvent;
	type VersionWrapper = ();
	// type WeightInfo = weights::cumulus_pallet_xcmp_queue::WeightInfo<Self>;
	type WeightInfo = ();
	// Enqueue XCMP messages from siblings for later processing.
	type XcmpQueue = frame_support::traits::TransformOrigin<
		MessageQueue,
		cumulus_primitives_core::AggregateMessageOrigin,
		cumulus_primitives_core::ParaId,
		message_queue::ParaIdToSibling,
	>;
}
