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

//! Darwinia economic inflation mechanism implementation.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unused_crate_dependencies)]
#![deny(missing_docs)]

#[cfg(test)]
mod test;

// crates.io
use primitive_types::U256;
// darwinia
use dc_types::{Balance, Moment, UNIT};
// github
use substrate_fixed::{
	transcendental,
	types::{I95F33, U94F34},
};

/// Inflation's upper limit.
pub const TOTAL_SUPPLY: Balance = 10_000_000_000 * UNIT;

/// Milliseconds per year.
pub const MILLISECS_PER_YEAR: Balance = (366 * 24 * 60 * 60) * 1000;

/// Compute the inflation of a period.
///
/// Use `U94F34` here, because `2^94 > TOTAL_SUPPLY * 10^9`.
pub fn in_period(unminted: Balance, period: Moment, elapsed: Moment) -> Option<Balance> {
	let unminted_per_millisecs = U94F34::from_num(unminted) / MILLISECS_PER_YEAR;
	let x = (unminted_per_millisecs * period).floor().to_num();
	let years = (elapsed / MILLISECS_PER_YEAR + 1) as _;

	inflate(x, years)
}

// Compute the inflation.
//
// Formula:
// ```
// x * (1 - (99 / 100) ^ sqrt(years));
// ```
//
// Use `I95F33` here, because `2^94 > TOTAL_SUPPLY * 10^9`.
fn inflate(x: Balance, years: u8) -> Option<Balance> {
	let sqrt = transcendental::sqrt::<I95F33, I95F33>(years.into()).ok()?;
	let ninety_nine = I95F33::from_num(99_u8) / 100_i128;
	let pow = transcendental::pow::<I95F33, I95F33>(ninety_nine, sqrt).ok()?;
	let ratio = I95F33::from_num(1_u8) - pow;
	let inflation = I95F33::from_num(x) * ratio;

	Some(inflation.floor().to_num())
}

/// Compute the reward of a deposit.
///
/// Reference(s):
/// - <https://github.com/evolutionlandorg/bank/blob/master/contracts/GringottsBank.sol#L280>
pub fn deposit_interest(amount: Balance, months: u8) -> Balance {
	let amount = U256::from(amount);
	let months = U256::from(months);
	let n = U256::from(67_u8).pow(months);
	let d = U256::from(66_u8).pow(months);
	let quot = n / d;
	let rem = n % d;
	let precision = U256::from(1_000_u16);

	// The result of `((quot - 1) * precision + rem * precision / d)` is `197` when months is `12`.
	//
	// The default interest is `1_000`.
	// So, we directly use `1_970_000` here instead `interest * 197 * 10^7`.
	(amount * (precision * (quot - 1_u8) + precision * rem / d) / 1_970_000_u32).as_u128()
}
