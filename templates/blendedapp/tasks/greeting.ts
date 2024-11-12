import { task } from "hardhat/config";

task("get-greeting", "Fetches the greeting from the deployed GreetingWithWorld contract")
  .addParam("contract", "The address of the deployed GreetingWithWorld contract")
  .setAction(async ({ contract }, hre) => {
    const { ethers } = hre;
    
    const getBlockNumber = await ethers.provider.getBlockNumber()
    console.log("Block Number:", getBlockNumber);

    const GreetingWithWorld = await ethers.getContractAt(
      "GreetingWithWorld",
      contract
    );

    const greeting = await GreetingWithWorld.getGreeting();
    // Convert the BigInt to a string for display
    console.log("Random Number:", greeting.toString());

    await GreetingWithWorld.updateCounter();
  });
