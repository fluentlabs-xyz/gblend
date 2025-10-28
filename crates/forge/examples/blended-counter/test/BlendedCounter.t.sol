// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";
import {BlendedCounter} from "../src/BlendedCounter.sol";
import {IPowerCalculator} from "../out/power-calculator.wasm/interface.sol";

contract BlendedCounterTest is Test {
    IPowerCalculator public powerCalculator;
    BlendedCounter public counter;

    function setUp() public {
        // Deploy the Rust-based PowerCalculator contract
        address calculatorAddr = vm.deployCode("out/power-calculator.wasm/foundry.json");
        powerCalculator = IPowerCalculator(calculatorAddr);

        // Deploy BlendedCounter with the PowerCalculator address
        counter = new BlendedCounter(calculatorAddr);
    }

    function testRustContractDirectly() public {
        // Call Rust contract directly via generated Solidity interface
        uint256 result = powerCalculator.power(2, 8);
        assertEq(result, 256, "Rust PowerCalculator returned wrong result");

        result = powerCalculator.power(3, 3);
        assertEq(result, 27, "Rust PowerCalculator returned wrong result");
    }

    function testIncrement() public {
        counter.increment();
        assertEq(counter.number(), 3); // starts at 2
    }

    function testSetNumber() public {
        counter.setNumber(42);
        assertEq(counter.number(), 42);
    }

    function testPowerThroughBlendedCounter() public {
        counter.setNumber(2);
        counter.power(3); // 2^3 = 8
        assertEq(counter.number(), 8);

        counter.setNumber(10);
        counter.power(2); // 10^2 = 100
        assertEq(counter.number(), 100);
    }
}
