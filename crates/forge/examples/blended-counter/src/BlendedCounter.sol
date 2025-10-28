// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {IPowerCalculator} from "../out/power-calculator.wasm/interface.sol";

/// @title BlendedCounter
/// @notice Example contract showing interaction between Solidity and Rust contracts.
contract BlendedCounter {
    uint256 public number;
    IPowerCalculator public immutable POWER_CALCULATOR;

    /// @param _powerCalculator Address of the deployed Rust PowerCalculator contract.
    constructor(address _powerCalculator) {
        POWER_CALCULATOR = IPowerCalculator(_powerCalculator);
        number = 2;
    }

    /// @notice Sets the counter to a new value.
    function setNumber(uint256 newNumber) public {
        number = newNumber;
    }

    /// @notice Increments the counter by 1.
    function increment() public {
        number++;
    }

    /// @notice Raises the current number to the given power using the Rust contract.
    /// @param exponent The exponent to raise the number to.
    function power(uint256 exponent) public {
        number = POWER_CALCULATOR.power(number, exponent);
    }
}
