require("@nomicfoundation/hardhat-ethers");
require("dotenv").config();

module.exports = {
  solidity: {
    // OpenZeppelin Contracts 5.6.x uses ^0.8.24 in the ERC721 and Strings
    // import graph and its Bytes helpers use the mcopy opcode available in
    // Solidity 0.8.27. Every repository contract currently declares ^0.8.20
    // or ^0.8.23, so 0.8.27 is a source-compatible common compiler.
    version: "0.8.27",
    settings: {
      evmVersion: "cancun",
      viaIR: true,
      optimizer: {
        enabled: true,
        runs: 200,
      },
    },
  },
  networks: {
    arbitrum: {
      url: "https://arb1.arbitrum.io/rpc",
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : [],
    },
    robinhood: {
      // Set ROBINHOOD_RPC_URL to an approved production RPC before deployment.
      url: process.env.ROBINHOOD_RPC_URL || "https://robinhood-rpc.publicnode.com",
      chainId: 4663,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : [],
      timeout: 120000,
    },
  },
};
