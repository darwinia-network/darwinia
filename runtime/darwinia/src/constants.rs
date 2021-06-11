// This file is part of Darwinia.
//
// Copyright (C) 2018-2021 Darwinia Network
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

//! A set of constant values used in Darwinia runtime.

/// Money matters.
pub mod currency {
	// --- darwinia ---
	use darwinia_primitives::{Balance, Power};

	pub const NANO: Balance = 1;
	pub const MICRO: Balance = 1_000 * NANO;
	pub const MILLI: Balance = 1_000 * MICRO;
	pub const COIN: Balance = 1_000 * MILLI;

	pub const CAP: Balance = 10_000_000_000 * COIN;
	pub const TOTAL_POWER: Power = 1_000_000_000;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 20 * MILLI + (bytes as Balance) * 100 * NANO
	}
}

/// Time and blocks.
pub mod time {
	// --- substrate ---
	use sp_staking::SessionIndex;
	// --- darwinia ---
	use darwinia_primitives::{BlockNumber, Moment};

	#[cfg(not(feature = "dev"))]
	pub const MILLISECS_PER_BLOCK: Moment = 6000;

	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

	#[cfg(feature = "dev")]
	pub const BLOCKS_PER_SESSION: BlockNumber = 10 * MINUTES;
	#[cfg(not(feature = "dev"))]
	pub const BLOCKS_PER_SESSION: BlockNumber = 4 * HOURS;

	#[cfg(feature = "dev")]
	pub const SESSIONS_PER_ERA: SessionIndex = 3;
	#[cfg(not(feature = "dev"))]
	pub const SESSIONS_PER_ERA: SessionIndex = 6;

	// These time units are defined in number of blocks.
	pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = 60 * MINUTES;
	pub const DAYS: BlockNumber = 24 * HOURS;

	// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
	pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
}

/// Fee-related.
pub mod fee {
	// --- crates ---
	use smallvec::smallvec;
	// --- substrate ---
	use frame_support::weights::{
		WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
	};
	use sp_runtime::Perbill;
	// --- darwinia ---
	use super::currency::*;
	use darwinia_primitives::Balance;
	use darwinia_runtime_common::ExtrinsicBaseWeight;

	/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
	/// node's balance type.
	///
	/// This should typically create a mapping between the following ranges:
	///   - [0, MAXIMUM_BLOCK_WEIGHT]
	///   - [Balance::min, Balance::max]
	///
	/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
	///   - Setting it to `0` will essentially disable the weight fee.
	///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
	pub struct WeightToFee;
	impl WeightToFeePolynomial for WeightToFee {
		type Balance = Balance;
		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
			// in Darwinia, extrinsic base weight (smallest non-zero weight) is mapped to 10 MILLI:
			let p = 10 * MILLI;
			let q = Balance::from(ExtrinsicBaseWeight::get());
			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}
}

pub mod relay {
	// --- darwinia ---
	use super::currency::*;
	use crate::*;
	use darwinia_relay_primitives::relayer_game::*;
	use ethereum_primitives::EthereumBlockNumber;

	pub struct EthereumRelayerGameAdjustor;
	impl AdjustableRelayerGame for EthereumRelayerGameAdjustor {
		type Moment = BlockNumber;
		type Balance = Balance;
		type RelayHeaderId = EthereumBlockNumber;

		fn max_active_games() -> u8 {
			32
		}

		fn affirm_time(round: u32) -> Self::Moment {
			match round {
				// 3 mins
				0 => 30,
				// 1.5 mins
				_ => 15,
			}
		}

		fn complete_proofs_time(_: u32) -> Self::Moment {
			// 3 mins
			30
		}

		fn update_sample_points(sample_points: &mut Vec<Vec<Self::RelayHeaderId>>) {
			if let Some(last_round_sample_points) = sample_points.last() {
				if let Some(last_sample_point) = last_round_sample_points.last() {
					let new_sample_points = vec![*last_sample_point - 1];

					sample_points.push(new_sample_points);
				} else {
					// Should never be reached
					log::error!(target: "ethereum-relayer-game", "Sample Round - NOT EXISTED");
				}
			} else {
				// Should never be reached
				log::error!(target: "ethereum-relayer-game", "Sample Point - NOT EXISTED");
			}
		}

		fn estimate_stake(round: u32, affirmations_count: u32) -> Self::Balance {
			match round {
				0 => match affirmations_count {
					0 => 100 * COIN,
					_ => 150 * COIN,
				},
				_ => 10 * COIN,
			}
		}
	}
}
