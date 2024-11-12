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
use dc_types::UNIT;
// github
use substrate_fixed::{transcendental, types::I95F33};

// Generate the issuing map.
//
// Formula:
// ```
// unissued * (1 - (99 / 100) ^ sqrt(years));
// ```
//
// Use `I95F33` here, because `2^94 > TOTAL_SUPPLY * 10^18`.
fn issuing_map() -> Vec<Balance> {
	let ninety_nine = I95F33::from_num(99_u8) / 100_i128;
	let max = 10_000_000_000_u128;
	let mut supply = 2_000_000_000_u128;

	(1_u8..=100)
		.map(|years| {
			let sqrt = transcendental::sqrt::<I95F33, I95F33>(years.into()).unwrap();
			let pow = transcendental::pow::<I95F33, I95F33>(ninety_nine, sqrt).unwrap();
			let ratio = I95F33::from_num(1_u8) - pow;
			let unissued = max - supply;
			let to_issue =
				(I95F33::checked_from_num(unissued).unwrap() * ratio).floor().to_num::<Balance>();

			supply += to_issue;

			to_issue * UNIT
		})
		.collect()
}

#[test]
fn issuing_map_should_work() {
	assert_eq!(issuing_map(), ISSUING_MAP);

	let max = 10_000_000_000_u128 * UNIT;
	let init = 2_000_000_000_u128 * UNIT;
	#[allow(clippy::approx_constant)]
	let rates = [
		4_f64, 5.37, 6.15, 6.56, 6.74, 6.76, 6.66, 6.5, 6.28, 6.04, 5.79, 5.52, 5.26, 4.99, 4.74,
		4.49, 4.25, 4.03, 3.81, 3.6, 3.4, 3.21, 3.04, 2.87, 2.71, 2.55, 2.41, 2.27, 2.14, 2.02,
		1.91, 1.8, 1.69, 1.59, 1.5, 1.41, 1.33, 1.25, 1.17, 1.1, 1.04, 0.97, 0.91, 0.86, 0.8, 0.75,
		0.71, 0.66, 0.62, 0.58, 0.54, 0.51, 0.47, 0.44, 0.41, 0.38, 0.36, 0.33, 0.31, 0.29, 0.27,
		0.25, 0.23, 0.21, 0.2, 0.18, 0.17, 0.16, 0.15, 0.14, 0.13, 0.12, 0.11, 0.1, 0.09, 0.08,
		0.08, 0.07, 0.07, 0.06, 0.06, 0.05, 0.05, 0.04, 0.04, 0.04, 0.03, 0.03, 0.03, 0.03, 0.02,
		0.02, 0.02, 0.02, 0.02, 0.01, 0.01, 0.01, 0.01, 0.01,
	];
	let mut unissued = max - init;

	rates.iter().zip(0..).for_each(|(rate, years)| {
		let issued = issuing_in_period(MILLISECS_PER_YEAR, years * MILLISECS_PER_YEAR).unwrap();

		sp_arithmetic::assert_eq_error_rate!(
			issued as f64 / (max - unissued) as f64,
			*rate / 100_f64,
			0.0001_f64
		);

		unissued -= issued;
	});
}
