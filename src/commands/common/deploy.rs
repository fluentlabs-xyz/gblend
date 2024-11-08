use crate::{
    commands::common::types::{NetworkConfig, NetworkType},
    error::Error,
    utils::wasm::validate_wasm,
};
use ethers::{
    core::types::{Bytes, TransactionReceipt as EthersReceipt, TransactionRequest, U256},
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::LocalWallet,
};
use std::{path::PathBuf, str::FromStr, sync::Arc, time::Duration};
use tokio::time::timeout;

/// Deploy a WASM contract to the specified network
pub async fn deploy_contract(
    network: &str,
    private_key: &str,
    gas_limit: u64,
    gas_price: u64,
    wasm_file: &PathBuf,
) -> Result<(), Error> {
    // Validate WASM file before deployment
    validate_wasm(wasm_file)?;

    // Get network configuration
    let network_config = get_network_config(network)?;

    // Validate private key and create wallet
    let wallet = validate_and_get_wallet(private_key)?;

    // Read WASM file
    let wasm_bytes = std::fs::read(wasm_file)
        .map_err(|e| Error::DeploymentError(format!("Failed to read WASM file: {}", e)))?;

    // Create provider
    let provider = Provider::<Http>::try_from(&network_config.endpoint)
        .map_err(|e| Error::NetworkError(format!("Failed to create provider: {}", e)))?;

    // Create client with wallet
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);

    println!("🔧 Preparing deployment transaction...");

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

    match timeout(
        Duration::from_secs(120),    // 2 minute timeout
        pending_tx.confirmations(3), // Wait for 3 confirmations
    )
    .await
    {
        Ok(receipt_result) => match receipt_result {
            Ok(Some(receipt)) => {
                print_deployment_info(&receipt);
                Ok(())
            }
            Ok(None) => Err(Error::DeploymentError(
                "Transaction receipt not found".to_string(),
            )),
            Err(e) => Err(Error::DeploymentError(format!("Transaction failed: {}", e))),
        },
        Err(_) => Err(Error::DeploymentError(
            "Transaction confirmation timed out".to_string(),
        )),
    }
}

fn get_network_config(network: &str) -> Result<NetworkConfig, Error> {
    match network {
        "local" => Ok(NetworkConfig {
            endpoint: "http://localhost:8545".to_string(),
            chain_id: 1337,
            network_type: NetworkType::Local,
        }),
        "dev" => Ok(NetworkConfig {
            endpoint: "https://testnet.example.com".to_string(),
            chain_id: 1,
            network_type: NetworkType::Dev,
        }),
        _ => Err(Error::NetworkError(format!("Unknown network: {}", network))),
    }
}

fn validate_and_get_wallet(private_key: &str) -> Result<LocalWallet, Error> {
    // Remove 0x prefix if present
    let clean_key = private_key.trim_start_matches("0x");

    // Validate key length (32 bytes = 64 hex chars)
    if clean_key.len() != 64 {
        return Err(Error::InvalidPrivateKey(
            "Private key must be 32 bytes (64 hex characters) long".to_string(),
        ));
    }

    // Try to parse the key
    let wallet = LocalWallet::from_str(clean_key)
        .map_err(|e| Error::InvalidPrivateKey(format!("Invalid private key format: {}", e)))?;

    Ok(wallet)
}

fn print_deployment_info(receipt: &EthersReceipt) {
    println!("\n✅ Contract deployed successfully!");
    if let Some(contract_address) = receipt.contract_address {
        println!("📍 Contract address: {}", contract_address);
    }
    println!("🧾 Transaction hash: {}", receipt.transaction_hash);
    println!("⛽ Gas used: {}", receipt.gas_used.unwrap_or_default());

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
