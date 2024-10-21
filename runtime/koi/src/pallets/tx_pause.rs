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

pub struct TxPauseWhitelistedCalls;
// This conflicts with runtime test.
#[cfg(feature = "runtime-benchmarks")]
impl frame_support::traits::Contains<pallet_tx_pause::RuntimeCallNameOf<Runtime>>
	for TxPauseWhitelistedCalls
{
	fn contains(_: &pallet_tx_pause::RuntimeCallNameOf<Runtime>) -> bool {
		false
	}
}
#[cfg(not(feature = "runtime-benchmarks"))]
impl frame_support::traits::Contains<pallet_tx_pause::RuntimeCallNameOf<Runtime>>
	for TxPauseWhitelistedCalls
{
	fn contains(full_name: &pallet_tx_pause::RuntimeCallNameOf<Runtime>) -> bool {
		let pallet = full_name.0.as_slice();

		// Pallets that can be paused by the tx-pause pallet.
		!matches!(
			pallet,
			b"Balances"
				| b"Assets" | b"Deposit"
				| b"DarwiniaStaking"
				| b"Ethereum"
				| b"EVM" | b"EthTxForwarder"
		)
	}
}

impl pallet_tx_pause::Config for Runtime {
	type MaxNameLen = ConstU32<256>;
	type PauseOrigin = RootOrAtLeastTwoThird<TechnicalCollective>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type UnpauseOrigin = RootOrAtLeastTwoThird<TechnicalCollective>;
	type WeightInfo = weights::pallet_tx_pause::WeightInfo<Runtime>;
	type WhitelistedCalls = TxPauseWhitelistedCalls;
}
