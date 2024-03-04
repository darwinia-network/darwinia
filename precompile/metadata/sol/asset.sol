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

/// @title ERC20Assets
/// notice The interface of ERC20Assets precompile
interface ERC20Assets {
	/// @dev Emitted when `value` tokens are moved from one account (`from`) to another (`to`). Note that `value` may be zero.
	/// @param from address The address sending the tokens
	/// @param to address The address receiving the tokens.
	/// @param value uint256 The amount of tokens transfered.
	event Transfer(address indexed from, address indexed to, uint256 value);

	/// @dev Emitted when the allowance of a `spender` for an `owner` is set by a call to {approve}. `value` is the new allowance.
    /// @param owner address Owner of the tokens.
    /// @param spender address Allowed spender.
    /// @param value uint256 Amount of tokens approved.
	event Approval(address indexed owner, address indexed spender, uint256 value);

	/// @dev Returns the name of the token.
	function name() external view returns (string memory);

	/// @dev Returns the symbol of the token.
	function symbol() external view returns (string memory);

	/// @dev Returns the decimals of the token.
	function decimals() external view returns (uint8);

	/// @dev Returns the amount of tokens in existence.
	function totalSupply() external returns (uint256);

	/// @dev Returns the amount of tokens owned by `account`.
	/// @param who The address to query the balance of.
	/// @return An uint256 representing the amount owned by the passed address.
	function balanceOf(address who) external returns (uint256);

	/// @dev Moves `amount` tokens from the caller's account to `to`.
	/// @param to The address to transfer to
	/// @param amount The amount to be transferred.
	/// @return returns true on success, false otherwise.
	function transfer(address to, uint256 amount) external returns (bool);

	/// @dev Returns the remaining number of tokens that `spender` will be allowed to spend on behalf of `owner` through {transferFrom}. This is zero by default.
	/// @param owner address The address which owns the funds.
	/// @param spender address The address which will spend the funds.
	/// @return A uint256 specifying the amount of tokens still available for the spender.
	function allowance(address owner, address spender) external view returns (uint256);

	/// @dev Sets `amount` as the allowance of `spender` over the caller's tokens.
	/// @param spender The address which will spend the funds.
	/// @param amount The amount of tokens to be spent.
	/// @return returns true on success, false otherwise.
	function approve(address spender, uint256 amount) external returns (bool);

	/// @dev Moves `amount` tokens from `from` to `to` using the allowance mechanism. `amount` is then deducted from the caller's allowance.
	/// @param from address The address which you want to send tokens from
	/// @param to address The address which you want to transfer to
	/// @param amount the amount of tokens to be transferred
	/// @return returns true on success, false otherwise.
	function transferFrom(address from, address to, uint256 amount) external returns (bool);
}
