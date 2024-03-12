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

//! Darwinia runtime constant collection.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unused_crate_dependencies)]
#![deny(missing_docs)]

// darwinia
use dc_primitives::{AccountId, Balance};
// frontier
use fp_account::AccountId20;
// substrate
use frame_support::PalletId;
use sp_std::prelude::*;

/// Existential deposit for Darwinia.
#[cfg(not(feature = "runtime-benchmarks"))]
pub const EXISTENTIAL_DEPOSIT: Balance = 0;
#[cfg(feature = "runtime-benchmarks")]
pub const EXISTENTIAL_DEPOSIT: Balance = 1;

/// An [`AccountId20`] generated from b"root".
pub const ROOT: AccountId20 =
	AccountId20([0x72, 0x6f, 0x6f, 0x74, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

frame_support::parameter_types! {
	/// Treasury account 0x6d6f646c64612f74727372790000000000000000.
	pub const TreasuryPid: PalletId = PalletId(*b"da/trsry");
	/// Existential deposit for Darwinia.
	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
	/// Maximum spending limit for root spender.
	pub const MaxBalance: Balance = Balance::max_value();
	/// Default asset creator for non-users.
	pub AssetCreators: Vec<AccountId> = vec![ROOT];
}
