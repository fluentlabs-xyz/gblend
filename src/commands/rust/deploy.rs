use crate::error::Error;
use clap::Args;
use ethers::{
    core::types::{Bytes, TransactionRequest, U256},
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
};
use std::{path::PathBuf, str::FromStr, sync::Arc, time::Duration};

#[derive(Args)]
pub struct DeployArgs {
    /// Private key for transaction signing (hex format with 0x prefix)
    #[arg(
        long,
        help = "Private key for transaction signing",
        long_help = "Private key in hex format with 0x prefix"
    )]
    private_key: String,

    /// Maximum gas limit for deployment transaction
    #[arg(
        long,
        help = "Maximum gas limit for deployment",
        default_value = "1000000"
    )]
    gas_limit: u64,

    /// Gas price in network's native currency units
    #[arg(
        long,
        help = "Gas price in network's native currency units",
        default_value = "1"
    )]
    gas_price: u64,

    /// Path to the WASM file for deployment
    #[arg(
        help = "Path to the WASM file for deployment",
        long_help = "Path to the compiled WASM file that will be deployed"
    )]
    wasm_file: PathBuf,
}

pub(super) async fn execute(args: &DeployArgs, network: &str) -> Result<(), Error> {
    // Validate and parse arguments
    validate_wasm_file(&args.wasm_file)?;
    let wallet = validate_and_get_wallet(&args.private_key)?;
    let network_config = get_network_config(network)?;

    // Deploy contract
    let receipt = deploy_contract(
        &args.wasm_file,
        wallet,
        &network_config,
        args.gas_limit,
        args.gas_price,
    )
    .await?;

    print_deployment_info(&receipt, network);
    Ok(())
}

#[derive(Debug)]
struct NetworkConfig {
    endpoint: String,
    chain_id: u64,
}

fn validate_wasm_file(wasm_file: &PathBuf) -> Result<(), Error> {
    if !wasm_file.exists() {
        return Err(Error::DeploymentError(format!(
            "WASM file not found: {}",
            wasm_file.display()
        )));
    }

    let wasm_bytes = std::fs::read(wasm_file)
        .map_err(|e| Error::DeploymentError(format!("Failed to read WASM file: {}", e)))?;

    // Check WASM magic number (0x6d736100)
    if wasm_bytes.len() < 4 || &wasm_bytes[0..4] != &[0x00, 0x61, 0x73, 0x6d] {
        return Err(Error::DeploymentError(
            "Invalid WASM file: missing magic number".to_string(),
        ));
    }

    Ok(())
}

fn validate_and_get_wallet(private_key: &str) -> Result<LocalWallet, Error> {
    // Remove 0x prefix if present
    let clean_key = private_key.trim_start_matches("0x");

    // Validate key length (32 bytes = 64 hex chars)
    if clean_key.len() != 64 {
        return Err(Error::DeploymentError(
            "Private key must be 32 bytes (64 hex characters) long".to_string(),
        ));
    }

    // Try to parse the key
    LocalWallet::from_str(clean_key)
        .map_err(|e| Error::DeploymentError(format!("Invalid private key format: {}", e)))
}

fn get_network_config(network: &str) -> Result<NetworkConfig, Error> {
    match network {
        "local" => Ok(NetworkConfig {
            endpoint: "http://localhost:8545".to_string(),
            chain_id: 1337,
        }),
        "dev" => Ok(NetworkConfig {
            endpoint: "https://testnet.example.com".to_string(),
            chain_id: 1,
        }),
        _ => Err(Error::NetworkError(format!("Unknown network: {}", network))),
    }
}

async fn deploy_contract(
    wasm_file: &PathBuf,
    wallet: LocalWallet,
    network_config: &NetworkConfig,
    gas_limit: u64,
    gas_price: u64,
) -> Result<ethers::core::types::TransactionReceipt, Error> {
    // Create provider
    let provider = Provider::<Http>::try_from(&network_config.endpoint)
        .map_err(|e| Error::NetworkError(format!("Failed to create provider: {}", e)))?;

    // Create client with wallet
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);

    println!("🔧 Preparing deployment transaction...");

    // Read WASM file
    let wasm_bytes = std::fs::read(wasm_file)
        .map_err(|e| Error::DeploymentError(format!("Failed to read WASM file: {}", e)))?;

    // Create deployment transaction
    let tx = TransactionRequest::new()
        .data(Bytes::from(wasm_bytes))
        .gas(U256::from(gas_limit))
        .gas_price(U256::from(gas_price));

    // Send transaction and wait for receipt
    println!("🚀 Sending transaction...");
    let pending_tx = client
        .send_transaction(tx, None)
        .await
        .map_err(|e| Error::DeploymentError(format!("Failed to send transaction: {}", e)))?;

    println!("⏳ Waiting for transaction confirmation...");

    let receipt = tokio::time::timeout(
        Duration::from_secs(120),    // 2 minute timeout
        pending_tx.confirmations(3), // Wait for 3 confirmations
    )
    .await
    .map_err(|_| Error::DeploymentError("Transaction confirmation timed out".to_string()))?
    .map_err(|e| Error::DeploymentError(format!("Transaction failed: {}", e)))?
    .ok_or_else(|| Error::DeploymentError("Transaction receipt not found".to_string()))?;

    Ok(receipt)
}

fn print_deployment_info(receipt: &ethers::core::types::TransactionReceipt, network: &str) {
    println!("\n✅ Contract deployed successfully!");
    if let Some(contract_address) = receipt.contract_address {
        println!("📍 Contract address: {}", contract_address);
    }
    println!("🌐 Network: {}", network);
    println!("🧾 Transaction hash: {}", receipt.transaction_hash);

    if let Some(gas_used) = receipt.gas_used {
        println!("⛽ Gas used: {}", gas_used);
    }

    if let Some(effective_gas_price) = receipt.effective_gas_price {
        println!("💰 Effective gas price: {}", effective_gas_price);
    }

    if let Some(block_number) = receipt.block_number {
        println!("🔲 Block number: {}", block_number);
    }

    if !receipt.logs.is_empty() {
        println!("📝 Events emitted: {}", receipt.logs.len());
    }
}
