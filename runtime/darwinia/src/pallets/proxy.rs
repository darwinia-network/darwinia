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

/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	MaxEncodedLen,
	TypeInfo,
	RuntimeDebug,
)]
pub enum ProxyType {
	#[codec(index = 0)]
	Any,
	#[codec(index = 1)]
	NonTransfer,
	#[codec(index = 2)]
	Governance,
	#[codec(index = 3)]
	Staking,
	// TODO: Migration.
	// #[codec(index = 4)]
	// IdentityJudgement,
	#[codec(index = 5)]
	CancelProxy,
	// #[codec(index = 6)]
	// EcdsaBridge,
	// #[codec(index = 7)]
	// SubstrateBridge,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}
impl frame_support::traits::InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => !matches!(
				c,
				RuntimeCall::Balances(..)
					| RuntimeCall::Assets(..)
					// Might contains transfer
					| RuntimeCall::Utility(..)
					| RuntimeCall::Proxy(..)
					| RuntimeCall::PolkadotXcm(..)
					| RuntimeCall::Ethereum(..)
			),
			ProxyType::Governance => matches!(
				c,
				RuntimeCall::ConvictionVoting(..)
					| RuntimeCall::Council(..)
					| RuntimeCall::Democracy(..)
					| RuntimeCall::Referenda(..)
					| RuntimeCall::TechnicalCommittee(..)
					| RuntimeCall::Treasury(..)
					| RuntimeCall::Whitelist(..)
			),
			ProxyType::Staking => {
				matches!(
					c,
					RuntimeCall::Session(..)
						| RuntimeCall::Deposit(..)
						| RuntimeCall::DarwiniaStaking(..)
				)
			},
			ProxyType::CancelProxy => {
				matches!(c, RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. }))
			},
		}
	}

	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::NonTransfer, _) => true,
			_ => false,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type AnnouncementDepositBase = ConstU128<{ darwinia_deposit(1, 8) }>;
	type AnnouncementDepositFactor = ConstU128<{ darwinia_deposit(0, 66) }>;
	type CallHasher = Hashing;
	type Currency = Balances;
	type MaxPending = ConstU32<32>;
	type MaxProxies = ConstU32<32>;
	// One storage item; key size 32, value size 8; .
	type ProxyDepositBase = ConstU128<{ darwinia_deposit(1, 8) }>;
	// Additional storage item size of 33 bytes.
	type ProxyDepositFactor = ConstU128<{ darwinia_deposit(0, 33) }>;
	type ProxyType = ProxyType;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	// type WeightInfo = weights::pallet_proxy::WeightInfo<Self>;
	type WeightInfo = ();
}
