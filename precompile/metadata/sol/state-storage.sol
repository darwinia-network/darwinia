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

/// @dev The StateStorage precompile address
address constant STATE_STORAGE_ADDRESS = 0x0000000000000000000000000000000000000400;

/// @dev The StateStorage contract instance
StateStorage constant STATE_STORAGE_CONTRACT = StateStorage(STATE_STORAGE_ADDRESS);

/// @title StateStorage
/// @notice The interface of the state storage precompile
interface StateStorage {
    /// @dev Get the storage value on a specific storage key, except EVM module.
    /// @param storageKey, the storage key follows the substrate storage mechanism.
    /// @return the storage value at the input key.
    function state_storage(bytes memory storageKey) external returns (bytes memory);
}
