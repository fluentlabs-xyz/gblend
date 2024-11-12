import { HardhatUserConfig } from "hardhat/types";
import "hardhat-deploy";
import "@nomicfoundation/hardhat-toolbox";
import "./tasks/greeting"

require("dotenv").config();

const DEPLOYER_PRIVATE_KEY = process.env.DEPLOYER_PRIVATE_KEY || "";

const config: HardhatUserConfig = {
  defaultNetwork: "localhost",
  networks: {
    localhost: {
      url: "https://rpc.dev.thefluent.xyz/",
      accounts: [DEPLOYER_PRIVATE_KEY],
    },
    dev: {
      url: "https://rpc.dev.thefluent.xyz/",
    },
    hardhat: {
      chainId: 1337,
    },
  },
  solidity: {
    version: "0.8.24",
    settings: {
      optimizer: {
        enabled: true,
        runs: 200,
      },
    },
  },
  namedAccounts: {
    deployer: {
      default: 0,
    },
  },
};

export default config;
