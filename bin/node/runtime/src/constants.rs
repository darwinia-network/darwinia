//! A set of constant values used in darwinia runtime.

/// Money matters.
pub mod currency {
	use node_primitives::Balance;

	pub const NANO: Balance = 1;
	pub const MICRO: Balance = 1_000 * NANO;
	pub const MILLI: Balance = 1_000 * MICRO;
	pub const COIN: Balance = 1_000 * MILLI;
}

/// Time.
pub mod time {
	use node_primitives::{BlockNumber, Moment};
	use sp_staking::SessionIndex;

	// TODO doc
	include!(concat!(env!("OUT_DIR"), "/timestamp_now.rs"));
	pub const GENESIS_TIME: Moment = NOW;

	/// Since BABE is probabilistic this is the average expected block time that
	/// we are targetting. Blocks will be produced at a minimum duration defined
	/// by `SLOT_DURATION`, but some slots will not be allocated to any
	/// authority and hence no block will be produced. We expect to have this
	/// block time on average following the defined slot duration and the value
	/// of `c` configured for BABE (where `1 - c` represents the probability of
	/// a slot being empty).
	/// This value is only used indirectly to define the unit constants below
	/// that are expressed in blocks. The rest of the code should use
	/// `SLOT_DURATION` instead (like the timestamp module for calculating the
	/// minimum period).
	///
	/// If using BABE with secondary slots (default) then all of the slots will
	/// always be assigned, in which case `MILLISECS_PER_BLOCK` and
	/// `SLOT_DURATION` should have the same value.
	///
	/// <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
	// Development
	pub const MILLISECS_PER_BLOCK: Moment = 6000;
	// Production
	// pub const MILLISECS_PER_BLOCK: Moment = 10000;
	pub const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;

	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

	// 1 in 4 blocks (on average, not counting collisions) will be primary BABE blocks.
	pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

	// Development
	pub const BLOCKS_PER_SESSION: BlockNumber = 3 * MINUTES;
	// Production
	// pub const BLOCKS_PER_SESSION: BlockNumber = 10 * MINUTES;
	pub const EPOCH_DURATION_IN_SLOTS: u64 = {
		const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

		(BLOCKS_PER_SESSION as f64 * SLOT_FILL_RATE) as u64
	};
	pub const SESSION_DURATION: BlockNumber = EPOCH_DURATION_IN_SLOTS as _;

	// Development
	pub const SESSIONS_PER_ERA: SessionIndex = 3;
	// Production
	// pub const SESSIONS_PER_ERA: SessionIndex = 6;

	// These time units are defined in number of blocks.
	pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = 60 * MINUTES;
	pub const DAYS: BlockNumber = 24 * HOURS;
}

pub mod supply {
	use crate::constants::currency::COIN;
	use node_primitives::{Balance, Power};

	pub const CAP: Balance = 1_000_000_000 * COIN;
	pub const TOTAL_POWER: Power = 1_000_000_000;
}
