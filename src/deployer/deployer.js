const { ethers } = require("ethers");
const fs = require("fs");

// Insert your private key 
const DEPLOYER_PRIVATE_KEY = '';

const main = async () => {
    // Check if a WASM binary path is provided
    if (process.argv.length < 3) {
        console.log(`You must specify the path to the WASM binary!`);
        console.log(`Example: node deployer.js --dev ../bin/greeting.wasm`);
        process.exit(-1);
    }

    // Read command-line arguments
    let args = process.argv.slice(2);
    const checkFlag = (param) => {
        let index = args.indexOf(param);
        if (index < 0) return false;
        args.splice(index, 1);
        return true;
    };
    let isLocal = checkFlag('--local');
    let isDev = checkFlag('--dev');

    // Set the provider URL based on the flag
    let providerUrl = isLocal ? 'http://127.0.0.1:8545' : isDev ? 'https://rpc.dev.thefluent.xyz/' : '';
    if (!providerUrl) {
        console.log(`You must specify --dev or --local flag!`);
        console.log(`Example: node deployer.js --dev ./bin/{yourfile}.wasm`);
        process.exit(-1);
    }

    // Read the WASM binary and convert it to hex
    let [binaryPath] = args;
    let wasmBinary;
    try {
        wasmBinary = fs.readFileSync(binaryPath).toString('hex');
    } catch (err) {
        console.error(`Failed to read the WASM file: ${err.message}`);
        process.exit(-1);
    }

    // Set up ethers.js provider and signer
    const provider = new ethers.JsonRpcProvider(providerUrl);
    const wallet = new ethers.Wallet(DEPLOYER_PRIVATE_KEY, provider);

    // Create a deployment transaction
    const tx = {
        data: '0x' + wasmBinary,
        gasLimit: 10_000_000,
    };

    // Send the transaction
    console.log('Deploying contract...');
    try {
        const txResponse = await wallet.sendTransaction(tx);
        console.log('Transaction sent:', txResponse.hash);

        // Wait for the transaction to be mined
        const receipt = await txResponse.wait();
        console.log('Transaction mined:', receipt);

        // Get the contract address from the receipt
        const contractAddress = receipt.contractAddress;
        console.log(`Contract deployed at: ${contractAddress}`);
    } catch (err) {
        console.error(`Deployment failed: ${err.message}`);
        process.exit(-1);
    }

    // Fetch the latest block number
    try {
        const latestBlockNumber = await provider.getBlockNumber();
        console.log(`Latest block number: ${latestBlockNumber}`);
    } catch (err) {
        console.error(`Failed to fetch the latest block number: ${err.message}`);
    }

    process.exit(0);
};

// Execute the main function
main().catch((err) => {
    console.error(err);
    process.exit(-1);
});
