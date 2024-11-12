// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./IFluentGreeting.sol";

contract GreetingWithWorld {
    IFluentGreeting public fluentGreetingContract;
    uint256 public counter;

    constructor(address _fluentGreetingContractAddress) {
        fluentGreetingContract = IFluentGreeting(_fluentGreetingContractAddress);
    }
    function updateCounter() external {
        counter++;
    }

    function getGreeting() external view returns (uint256) {
        // Call the random function from the fluentGreetingContract
        uint256 randi = fluentGreetingContract.random();
        
        // Return the greeting value
        return randi;
    }
}
