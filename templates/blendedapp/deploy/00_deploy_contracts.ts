import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction } from "hardhat-deploy/types";
import { ethers } from "ethers";
import fs from "fs";
import crypto from "crypto";
import path from "path";
require("dotenv").config();

const DEPLOYER_PRIVATE_KEY =
  process.env.DEPLOYER_PRIVATE_KEY ||
  "";

const func: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { deployments, getNamedAccounts, ethers, config, network } = hre;
  const { deploy, save, getOrNull } = deployments;
  const { deployer: deployerAddress } = await getNamedAccounts();

  console.log("deployerAddress", deployerAddress);
  // ---------------------
  // Deploy WASM contract
  // ---------------------
  console.log("Deploying WASM contract...");
  const wasmBinaryPath = "./hellorust/bin/greeting.wasm"; // TODO: Update this path to your actual wasm file
  // @ts-ignore
  const provider = new ethers.JsonRpcProvider(network.config.url);

  // Create a new wallet instance to deploy the contract,
  // as using the default wallet enforces a maximum contract size limit,
  // which is not applicable in our case.
  const deployer = new ethers.Wallet(DEPLOYER_PRIVATE_KEY, provider);

  const checkmateValidatorAddress = await deployWasmContract(
    wasmBinaryPath,
    deployer,
    provider,
    getOrNull,
    save
  );


  // ---------------------
  // Deploy GreetingWithWorld contract
  // ---------------------
  console.log("Deploying GreetingWithWorld contract...");
  const fluentGreetingContractAddress = checkmateValidatorAddress; // Replace with the actual address if different

  const greetingWithWorld = await deploy("GreetingWithWorld", {
    from: deployerAddress,
    args: [fluentGreetingContractAddress],
    log: true,
  });

  console.log(`GreetingWithWorld contract deployed at: ${greetingWithWorld.address}`);
};


async function deployWasmContract(
    wasmBinaryPath: string,
    deployer: ethers.Wallet,
    provider: ethers.JsonRpcProvider,
    getOrNull: any,
    save: any
  ) {
    const wasmBinary = fs.readFileSync(wasmBinaryPath);
    const wasmBinaryHash = crypto
      .createHash("sha256")
      .update(wasmBinary)
      .digest("hex");
    const artifactName = path.basename(wasmBinaryPath, ".wasm");
    const existingDeployment = await getOrNull(artifactName);
  
    if (existingDeployment && existingDeployment.metadata === wasmBinaryHash) {
      console.log(`WASM contract bytecode has not changed. Skipping deployment.`);
      console.log(`Existing contract address: ${existingDeployment.address}`);
      return existingDeployment.address;
    }
  
    const gasPrice = (await provider.getFeeData()).gasPrice;
  
    const transaction = {
      data: "0x" + wasmBinary.toString("hex"),
      gasLimit: 500_000,
      gasPrice: gasPrice,
    };
  
    const tx = await deployer.sendTransaction(transaction);
    const receipt = await tx.wait();
  
    if (receipt && receipt.contractAddress) {
      console.log(`WASM Greeting contract deployed at: ${receipt.contractAddress}`);
  
      const artifact = {
        abi: [], // Since there's no ABI for the WASM contract
        bytecode: "0x" + wasmBinary.toString("hex"),
        deployedBytecode: "0x" + wasmBinary.toString("hex"),
        metadata: wasmBinaryHash,
      };
  
      const deploymentData = {
        address: receipt.contractAddress,
        ...artifact,
      };
  
      await save(artifactName, deploymentData);
    } else {
      throw new Error("Failed to deploy WASM contract");
    }
  
    return receipt.contractAddress;
  }

export default func;
func.tags = ["all"];
