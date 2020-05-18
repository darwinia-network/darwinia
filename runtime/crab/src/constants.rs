//! A set of constant values used in Crab runtime.

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
}

/// Time and blocks.
pub mod time {
	// --- substrate ---
	use sp_staking::SessionIndex;
	// --- darwinia ---
	use darwinia_primitives::{BlockNumber, Moment};

	// Mainnet
	// pub const MILLISECS_PER_BLOCK: Moment = 10000;
	// Crab & Testnet
	pub const MILLISECS_PER_BLOCK: Moment = 6000;

	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

	// Mainnet
	// pub const BLOCKS_PER_SESSION: BlockNumber = 4 * HOURS;
	// Crab
	pub const BLOCKS_PER_SESSION: BlockNumber = 1 * HOURS;
	// Testnet
	// pub const BLOCKS_PER_SESSION: BlockNumber = 10 * MINUTES;

	// Crab & Mainnet
	pub const SESSIONS_PER_ERA: SessionIndex = 6;
	// Testnet
	// pub const SESSIONS_PER_ERA: SessionIndex = 3;

	// These time units are defined in number of blocks.
	pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = 60 * MINUTES;
	pub const DAYS: BlockNumber = 24 * HOURS;

	// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
	pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
}

/// Fee-related.
pub mod fee {
	// --- substrate ---
	use frame_support::weights::Weight;
	use sp_runtime::{traits::Convert, Perbill};
	// --- darwinia ---
	use super::currency::MILLI;
	use darwinia_primitives::Balance;

	/// The block saturation level. Fees will be updates based on this value.
	pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

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
	impl Convert<Weight, Balance> for WeightToFee {
		fn convert(x: Weight) -> Balance {
			// in Crab a weight of 10_000_000 (smallest non-zero weight) is mapped to 1/10 MILLI:
			Balance::from(x).saturating_mul(MILLI / (10 * 10_000_000))
		}
	}
}
