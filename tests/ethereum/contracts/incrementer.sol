// SPDX-License-Identifier: GPL-3.0

pragma solidity ^0.8.0;

contract Incrementer {
    uint256 public number;
    event Increment(address indexed sender, uint256 indexed value);

    constructor(uint256 _initialNumber) {
        number = _initialNumber;
    }

    function increment(uint256 _value) public {
        if (_value < 3) {
            revert("the value must be greater than 3");
        }
        number = number + _value;
        emit Increment(msg.sender, _value);
    }

    function reset() public {
        number = 0;
    }
}