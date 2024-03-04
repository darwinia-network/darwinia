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

/// @dev The BLS precompile address
address constant BLS_ADDRESS = 0x0000000000000000000000000000000000000800;

/// @dev The Bls contract instance
BLS12381 constant BLS_CONTRACT = BLS12381(BLS_ADDRESS);

/// @title BLS12381
/// @notice The interface of BLS12381 precompile
interface BLS12381 {
	/// @dev Verifies an aggregate_signature against a list of pub_keys.
	/// @param pubKeys, trusted public keys
	/// @param message, the message to be signed.
	/// @param signature, the signature to ve verified.
	/// @return returns true on success, false otherwise.
	function fast_aggregate_verify(
		bytes[] memory pubKeys,
		bytes memory message,
		bytes memory signature
	) external returns (bool);
}
