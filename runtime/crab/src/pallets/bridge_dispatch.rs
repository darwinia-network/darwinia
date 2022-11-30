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

pub use pallet_bridge_dispatch::Instance1 as WithDarwiniaDispatch;

// darwinia
use crate::*;
use bp_message_dispatch::{Everything, IntoDispatchOrigin as IntoDispatchOriginT};
use bp_messages::{LaneId, MessageNonce};
use pallet_bridge_dispatch::Config;

pub struct IntoDispatchOrigin;
impl IntoDispatchOriginT<bp_crab::AccountId, RuntimeCall, RuntimeOrigin> for IntoDispatchOrigin {
	fn into_dispatch_origin(id: &bp_crab::AccountId, _: &RuntimeCall) -> RuntimeOrigin {
		frame_system::RawOrigin::Signed(id.clone()).into()
	}
}

impl Config<WithDarwiniaDispatch> for Runtime {
	type AccountIdConverter = bp_crab::AccountIdConverter;
	type BridgeMessageId = (LaneId, MessageNonce);
	type CallValidator = Everything;
	type EncodedCall = bm_darwinia::FromDarwiniaEncodedCall;
	type IntoDispatchOrigin = IntoDispatchOrigin;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type SourceChainAccountId = bp_darwinia::AccountId;
	type TargetChainAccountPublic = bp_crab::AccountPublic;
	type TargetChainSignature = bp_crab::Signature;
}
