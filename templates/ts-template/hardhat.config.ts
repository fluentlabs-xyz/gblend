import { HardhatUserConfig } from "hardhat/config";
import "@nomiclabs/hardhat-ethers";
import "@nomiclabs/hardhat-vyper";

const config: HardhatUserConfig = {
  networks: {
    fluent_devnet1: {
      url: "https://rpc.dev.gblend.xyz/",
      chainId: 20993,
      accounts: [
        `0x${"ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"}`,
      ],
    },
  },
  solidity: {
    version: "0.8.19",
  },
};

export default config;
