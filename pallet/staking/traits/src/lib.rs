#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
#![deny(unused_crate_dependencies)]

//! # Darwinia parachain staking's traits

// core
use core::fmt::Debug;
// crates.io
use codec::{FullCodec, MaxEncodedLen};
use scale_info::TypeInfo;
// substrate
use sp_runtime::{DispatchError, DispatchResult};

/// Stake trait that stake items must be implemented.
pub trait Stake {
	/// Account type.
	type AccountId;
	/// Stake item type.
	///
	/// Basically, it's just a num type.
	#[cfg(not(feature = "runtime-benchmarks"))]
	type Item: Clone + Copy + Debug + PartialEq + FullCodec + MaxEncodedLen + TypeInfo;
	/// Stake item type.
	///
	/// Basically, it's just a num type.
	#[cfg(feature = "runtime-benchmarks")]
	type Item: Clone
		+ Copy
		+ Debug
		+ Default
		+ From<u8>
		+ PartialEq
		+ FullCodec
		+ MaxEncodedLen
		+ TypeInfo;

	/// Add stakes to the staking pool.
	///
	/// This will transfer the stakes to a pallet/contact account.
	fn stake(who: &Self::AccountId, item: Self::Item) -> DispatchResult;

	/// Withdraw stakes from the staking pool.
	///
	/// This will transfer the stakes back to the staker's account.
	fn unstake(who: &Self::AccountId, item: Self::Item) -> DispatchResult;
}

/// Extended stake trait.
///
/// Provide a way to access the deposit RING amount.
pub trait StakeExt: Stake {
	/// Amount type.
	type Amount;

	/// Get the staked amount.
	fn amount(who: &Self::AccountId, item: Self::Item) -> Result<Self::Amount, DispatchError>;
}
