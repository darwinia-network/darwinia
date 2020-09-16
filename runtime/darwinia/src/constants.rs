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

	pub const RING_EXISTENTIAL_DEPOSIT: u128 = 100 * MICRO;
	pub const KTON_EXISTENTIAL_DEPOSIT: u128 = MICRO;

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

	#[cfg(feature = "dev")]
	pub const MILLISECS_PER_BLOCK: Moment = 3000;
	#[cfg(not(feature = "dev"))]
	pub const MILLISECS_PER_BLOCK: Moment = 6000;

	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

	#[cfg(feature = "dev")]
	pub const BLOCKS_PER_SESSION: BlockNumber = MINUTES / 2;
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
	///   - [0, system::MaximumBlockWeight]
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
				coeff_frac: Perbill::from_rational_approximation(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}
}

pub mod relay {
	// --- darwinia ---
	use super::currency::*;
	use crate::*;
	use darwinia_relay_primitives::*;

	pub struct EthereumRelayerGameAdjustor;
	impl AdjustableRelayerGame for EthereumRelayerGameAdjustor {
		type Moment = BlockNumber;
		type Balance = Balance;
		type TcBlockNumber = <<EthereumRelay as Relayable>::HeaderThing as HeaderThing>::Number;

		fn challenge_time(round: Round) -> Self::Moment {
			match round {
				// 3 mins
				0 => 30,
				// 1 mins
				_ => 10,
			}
		}

		fn round_of_samples_count(chain_len: u64) -> Round {
			chain_len - 1
		}

		fn samples_count_of_round(round: Round) -> u64 {
			round + 1
		}

		fn update_samples(samples: &mut Vec<Vec<Self::TcBlockNumber>>) {
			samples.push(vec![samples.last().unwrap().last().unwrap() - 1]);
		}

		fn estimate_bond(round: Round, proposals_count: u64) -> Self::Balance {
			match round {
				0 => match proposals_count {
					0 => 1000 * COIN,
					_ => 1500 * COIN,
				},
				_ => 100 * COIN,
			}
		}
	}
}
