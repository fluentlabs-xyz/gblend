require("@nomiclabs/hardhat-ethers");
require("@nomiclabs/hardhat-vyper");
/**
 * @type import('hardhat/config').HardhatUserConfig
 */
module.exports = {
  networks: {
    fluent_devnet1: {
      url: "https://rpc.dev.gblend.xyz/",
      chainId: 20993,
      accounts: [
        `0x${"ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"}`,
      ], // Replace with the private key of the deploying account
    },
  },
  solidity: {
    version: "0.8.19",
  },
  vyper: {
    version: "0.3.0",
  },
};
