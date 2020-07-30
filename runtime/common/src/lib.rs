//! Common runtime code for Darwinia and Crab.

#![cfg_attr(not(feature = "std"), no_std)]

/// Implementations of some helper traits passed into runtime modules as associated types.
pub mod impls;
pub use impls::*;

// --- substrate ---
pub use frame_support::weights::constants::{
	BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight,
};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
// --- darwinia ---
#[cfg(feature = "std")]
pub use darwinia_staking::StakerStatus;

// --- crates ---
use static_assertions::const_assert;
// --- substrate ---
use frame_support::{
	parameter_types,
	traits::Currency,
	weights::{constants::WEIGHT_PER_SECOND, Weight},
};
use pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment};
use sp_runtime::{traits::Saturating, FixedPointNumber, Perbill, Perquintill};
// --- darwinia ---
use darwinia_primitives::BlockNumber;

pub type RingInstance = darwinia_balances::Instance0;
pub type KtonInstance = darwinia_balances::Instance1;

pub type NegativeImbalance<T> = <darwinia_balances::Module<T, RingInstance> as Currency<
	<T as frame_system::Trait>::AccountId,
>>::NegativeImbalance;

/// We assume that an on-initialize consumes 10% of the weight on average, hence a single extrinsic
/// will not be allowed to consume more than `AvailableBlockRatio - 10%`.
const AVERAGE_ON_INITIALIZE_WEIGHT: Perbill = Perbill::from_percent(10);
parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	/// Block time that can be used by weights.
	pub const MaximumBlockWeight: Weight = 2 * WEIGHT_PER_SECOND;
	/// Maximum weight that a _single_ extrinsic can take.
	pub MaximumExtrinsicWeight: Weight = AvailableBlockRatio::get()
		.saturating_sub(AVERAGE_ON_INITIALIZE_WEIGHT) * MaximumBlockWeight::get();
	pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
	/// Portion of the block available to normal class of dispatches.
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	/// The portion of the `AvailableBlockRatio` that we adjust the fees with. Blocks filled less
	/// than this will decrease the weight and more will increase.
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	/// The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
	/// change the fees more rapidly.
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(3, 100_000);
	/// Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
	/// that combined with `AdjustmentVariable`, we can recover from the minimum.
	/// See `multiplier_can_grow_from_zero`.
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
}
const_assert!(
	AvailableBlockRatio::get().deconstruct() >= AVERAGE_ON_INITIALIZE_WEIGHT.deconstruct()
);

/// Parameterized slow adjusting fee updated based on
/// https://w3f-research.readthedocs.io/en/latest/polkadot/Token%20Economics.html#-2.-slow-adjusting-mechanism
pub type SlowAdjustingFeeUpdate<R> =
	TargetedFeeAdjustment<R, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;
