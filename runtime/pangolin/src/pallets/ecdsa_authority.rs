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

const MAX_PENDING_PERIOD: BlockNumber = 100;
const SYNC_INTERVAL: BlockNumber = 10;

frame_support::parameter_types! {
	pub const SignThreshold: sp_runtime::Perbill = sp_runtime::Perbill::from_percent(60);
}
static_assertions::const_assert!(MAX_PENDING_PERIOD > SYNC_INTERVAL);

impl darwinia_ecdsa_authority::Config for Runtime {
	type ChainId = <Self as pallet_evm::Config>::ChainId;
	type MaxAuthorities = ConstU32<7>;
	type MaxPendingPeriod = ConstU32<MAX_PENDING_PERIOD>;
	type MessageRoot = darwinia_message_gadget::MessageRootGetter<Self>;
	type RuntimeEvent = RuntimeEvent;
	type SignThreshold = SignThreshold;
	type SyncInterval = ConstU32<SYNC_INTERVAL>;
	type WeightInfo = weights::darwinia_ecdsa_authority::WeightInfo<Self>;
}
