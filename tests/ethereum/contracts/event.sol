// SPDX-License-Identifier: GPL-3.0

pragma solidity ^0.6.0;

contract JSON_Test {
    event Log0(uint256 value);
    event Log0Anonym(uint256 value) anonymous;

    event Log1(bool indexed aBool, uint256 value);
    event Log1Anonym(bool indexed aBool, uint256 value) anonymous;

    event Log2(bool indexed aBool, address indexed aAddress, uint256 value);
    event Log2Anonym(
        bool indexed aBool,
        address indexed aAddress,
        uint256 value
    ) anonymous;

    event Log3(
        bool indexed aBool,
        address indexed aAddress,
        bytes32 indexed aBytes32,
        uint256 value
    );
    event Log3Anonym(
        bool indexed aBool,
        address indexed aAddress,
        bytes32 indexed aBytes32,
        uint256 value
    ) anonymous;

    constructor() public {}

    function fireEventLog0() public {
        emit Log0(42);
    }

    function fireEventLog0Anonym() public {
        emit Log0Anonym(42);
    }

    function fireEventLog1() public {
        emit Log1(true, 42);
    }

    function fireEventLog1Anonym() public {
        emit Log1Anonym(true, 42);
    }

    function fireEventLog2() public {
        emit Log2(true, msg.sender, 42);
    }

    function fireEventLog2Anonym() public {
        emit Log2Anonym(true, msg.sender, 42);
    }

    function fireEventLog3() public {
        emit Log3(
            true,
            msg.sender,
            0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff,
            42
        );
    }

    function fireEventLog3Anonym() public {
        emit Log3Anonym(
            true,
            msg.sender,
            0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff,
            42
        );
    }
}
