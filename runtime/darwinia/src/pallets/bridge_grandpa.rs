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

pub use pallet_bridge_grandpa::Instance1 as WithCrabGrandpa;

// darwinia
use crate::*;
use pallet_bridge_grandpa::Config;

frame_support::parameter_types! {
	// This is a pretty unscientific cap.
	//
	// Note that once this is hit the pallet will essentially throttle incoming requests down to one
	// call per block.
	pub const MaxRequests: u32 = 50;
	pub const CrabHeadersToKeep: u32 = 500;
}

impl Config<WithCrabGrandpa> for Runtime {
	type BridgedChain = bp_crab::DarwiniaLike;
	type HeadersToKeep = CrabHeadersToKeep;
	type MaxBridgedAuthorities = ();
	type MaxBridgedHeaderSize = ();
	type MaxRequests = MaxRequests;
	type WeightInfo = ();
}
