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

pragma solidity >=0.8.3;

/// @dev The Deposit precompile address
address constant DEPOSIT_ADDRESS = 0x0000000000000000000000000000000000000600;

/// @dev The Desposit contract instance
Deposit constant DEPOSIT_CONTRACT = Deposit(DEPOSIT_ADDRESS);

/// @title Deposit
/// @notice The interface of the darwinia deposit pallet.
interface Deposit {
	/// @dev Lock the RING for some KTON profit/interest.
	/// @param ringAmount, the amount of the RING asset
	/// @param months, the lock time 1 ~ 36
	/// @return returns true on success, false otherwise.
	function lock(uint256 ringAmount, uint8 months) external returns (bool);

	/// @dev Claim the expired-locked RING.
	/// @return returns true on success, false otherwise.
	function claim() external returns (bool);

	/// @dev Claim the unexpired-locked RING by paying the KTON penalty.
	/// @param depositId The deposit ticket wish to claim
	/// @return returns true on success, false otherwise.
	function claim_with_penalty(uint8 depositId) external returns (bool);

	/// @dev Migrate data to deposit contract
	function migrate() external returns (bool);
}
