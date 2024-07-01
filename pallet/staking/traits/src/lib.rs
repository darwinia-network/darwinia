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

//! # Darwinia parachain staking's traits

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

// core
use core::fmt::Debug;
// crates.io
use codec::{FullCodec, MaxEncodedLen};
use scale_info::TypeInfo;
// polkadot-sdk
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
		+ From<u16>
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
