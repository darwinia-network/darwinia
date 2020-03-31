//! Common runtime code for Darwinia and Crab.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod impls;
/// Implementations of some helper traits passed into runtime modules as associated types.
pub use impls::{support_kton_in_the_future, AccountData, TargetedFeeAdjustment, ToAuthor};

// --- substrate ---
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
// --- darwinia ---
#[cfg(feature = "std")]
pub use darwinia_staking::StakerStatus;

// --- substrate ---
use frame_support::{parameter_types, traits::Currency, weights::Weight};
use sp_runtime::Perbill;
// --- darwinia ---
use node_primitives::BlockNumber;

pub type RingInstance = darwinia_balances::Instance0;
pub type KtonInstance = darwinia_balances::Instance1;

pub type NegativeImbalance<T> = <darwinia_balances::Module<T, RingInstance> as Currency<
	<T as frame_system::Trait>::AccountId,
>>::NegativeImbalance;

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	pub const MaximumBlockWeight: Weight = 1_000_000_000;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
}
