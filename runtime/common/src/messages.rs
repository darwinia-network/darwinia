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

// darwinia
use crate::system::*;

/// Convert a 256-bit hash into an AccountId.
pub struct AccountIdConverter;
impl sp_runtime::traits::Convert<sp_core::H256, dc_primitives::AccountId> for AccountIdConverter {
	fn convert(hash: sp_core::H256) -> dc_primitives::AccountId {
		// This way keep compatible with darwinia 1.0 substrate to evm account rule.
		let evm_address = sp_core::H160::from_slice(&hash.as_bytes()[0..20]);
		evm_address.into()
	}
}

/// Darwinia-like chain.
pub struct DarwiniaLike;
impl bp_runtime::Chain for DarwiniaLike {
	type AccountId = dc_primitives::AccountId;
	type Balance = dc_primitives::Balance;
	type BlockNumber = dc_primitives::BlockNumber;
	type Hash = dc_primitives::Hash;
	type Hasher = dc_primitives::Hashing;
	type Header = dc_primitives::Header;
	type Index = dc_primitives::Nonce;
	type Signature = dc_primitives::Signature;

	fn max_extrinsic_size() -> u32 {
		*RuntimeBlockLength::get().max.get(frame_support::dispatch::DispatchClass::Normal)
	}

	fn max_extrinsic_weight() -> frame_support::weights::Weight {
		RuntimeBlockWeights::get()
			.get(frame_support::dispatch::DispatchClass::Normal)
			.max_extrinsic
			.unwrap_or(frame_support::weights::Weight::MAX)
	}
}

frame_support::parameter_types! {
	/// Maximal number of unconfirmed messages at inbound lane.
	pub const MaxUnconfirmedMessagesAtInboundLane: bp_messages::MessageNonce = 8192;
	/// Maximal number of unrewarded relayer entries at inbound lane.
	pub const MaxUnrewardedRelayerEntriesAtInboundLane: bp_messages::MessageNonce = 128;
}
