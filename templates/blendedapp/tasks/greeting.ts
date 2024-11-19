import { task } from "hardhat/config";

task(
  "get-greeting",
  "Fetches the greeting from the deployed GreetingWithWorld contract"
)
  .addParam(
    "contract",
    "The address of the deployed GreetingWithWorld contract"
  )
  .setAction(async ({ contract }, hre) => {
    const { ethers } = hre;
    const GreetingWithWorld = await ethers.getContractAt(
      "GreetingWithWorld",
      contract
    );
    const greeting = await GreetingWithWorld.getGreeting();
    console.log("Greeting:", greeting);
  });
