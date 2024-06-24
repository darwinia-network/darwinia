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

//! Darwinia core types.

#![no_std]
#![deny(missing_docs)]
// #![deny(unused_crate_dependencies)]

/// Balance type.
pub type Balance = u128;

/// Time type.
pub type Moment = u128;

/// Asset identifier type.
pub type AssetId = u64;

// Unit = the base number of indivisible units for balances
/// 1e18 wei — 1,000,000,000,000,000,000
pub const UNIT: Balance = 1_000 * MILLIUNIT;
/// 1e15 wei — 1,000,000,000,000,000
pub const MILLIUNIT: Balance = 1_000 * MICROUNIT;
/// 1e12 wei — 1,000,000,000,000
pub const MICROUNIT: Balance = 1_000 * GWEI;
/// 1e9 wei — 1,000,000,000
pub const GWEI: Balance = 1_000 * MWEI;
/// 1e6 wei — 1,000,000
pub const MWEI: Balance = 1_000 * KWEI;
/// 1e3 wei — 1,000
pub const KWEI: Balance = 1_000 * WEI;
/// 1 wei — 1
pub const WEI: Balance = 1;
