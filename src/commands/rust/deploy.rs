use crate::error::Error;
use clap::Args;
use core::fmt;
use ethers::{
    core::types::{Bytes, TransactionRequest, U256},
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
};
use std::{path::PathBuf, str::FromStr, sync::Arc};

const DEFAULT_GAS_LIMIT: u64 = 30_000_000;
const DEFAULT_GAS_PRICE: u64 = 0;

#[derive(Args)]
pub struct DeployArgs {
    #[arg(
        long,
        help = "Private key for transaction signing in hex format (0x prefix)",
        env = "DEPLOY_PRIVATE_KEY"
    )]
    private_key: String,

    #[arg(
        long,
        help = "Maximum gas limit for deployment",
        default_value_t = DEFAULT_GAS_LIMIT,
        env = "DEPLOY_GAS_LIMIT"
    )]
    gas_limit: u64,

    #[arg(
        long,
        help = "Optional gas price in network's native units. Fetched from network if zero.",
        default_value_t = DEFAULT_GAS_PRICE,
        env = "DEPLOY_GAS_PRICE"

    )]
    gas_price: u64,

    #[arg(help = "Path to the compiled WASM file for deployment")]
    wasm_file: PathBuf,

    /// Use the local network configuration
    #[arg(long, help = "Use the local network configuration")]
    pub local: bool,

    /// Use the development network configuration
    #[arg(long, help = "Use the development network configuration")]
    pub dev: bool,

    /// Custom RPC endpoint for network configuration
    #[arg(long, help = "Custom RPC endpoint", conflicts_with_all = &["local", "dev"])]
    pub rpc: Option<String>,

    /// Custom chain ID for network configuration
    #[arg(long, help = "Custom chain ID", conflicts_with_all = &["local", "dev"])]
    pub chain_id: Option<u64>,
}

pub(super) async fn execute(args: &DeployArgs) -> Result<(), Error> {
    validate_wasm_file(&args.wasm_file)?;
    let wallet = create_wallet(&args.private_key)?;

    // Determine the network configuration
    let network_config = NetworkConfig::from_args(args)?;

    // Deploy the contract
    let receipt = deploy_contract(
        &args.wasm_file,
        wallet,
        &network_config,
        args.gas_limit,
        args.gas_price,
    )
        .await?;

    print_deployment_info(&receipt, &network_config);
    Ok(())
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
    if wasm_bytes.len() < 4 || &wasm_bytes[0..4] != &[0x00, 0x61, 0x73, 0x6d] {
        return Err(Error::DeploymentError(
            "Invalid WASM file: missing magic number".to_string(),
        ));
    }
    Ok(())
}

fn create_wallet(private_key: &str) -> Result<LocalWallet, Error> {
    let clean_key = private_key.trim_start_matches("0x");
    if clean_key.len() != 64 {
        return Err(Error::DeploymentError(
            "Private key must be 64 hex characters.".to_string(),
        ));
    }
    LocalWallet::from_str(clean_key)
        .map_err(|e| Error::DeploymentError(format!("Invalid private key: {}", e)))
}

async fn deploy_contract(
    wasm_file: &PathBuf,
    wallet: LocalWallet,
    network_config: &NetworkConfig,
    gas_limit: u64,
    gas_price: u64,
) -> Result<ethers::core::types::TransactionReceipt, Error> {
    let wallet = wallet.with_chain_id(network_config.chain_id);

    let provider = Provider::<Http>::try_from(&network_config.endpoint)
        .map_err(|e| Error::NetworkError(format!("Failed to create provider: {}", e)))?;
    let client = Arc::new(SignerMiddleware::new(provider.clone(), wallet));

    println!("🔧 Preparing deployment transaction...");

    let gas_price = if gas_price == 0 {
        provider
            .get_gas_price()
            .await
            .map_err(|e| Error::NetworkError(format!("Failed to fetch gas price: {}", e)))?
    } else {
        U256::from(gas_price)
    };

    let wasm_bytes = std::fs::read(wasm_file)
        .map_err(|e| Error::DeploymentError(format!("Failed to read WASM file: {}", e)))?;

    let tx = TransactionRequest {
        chain_id: Some(network_config.chain_id.into()),
        data: Some(Bytes::from(wasm_bytes)),
        gas: Some(U256::from(gas_limit)),
        gas_price: Some(gas_price),
        ..Default::default()
    };

    println!("🚀 Sending transaction...");
    let pending_tx = client
        .send_transaction(tx, None)
        .await
        .map_err(|e| Error::DeploymentError(format!("Failed to send transaction: {}", e)))?;

    let receipt = pending_tx
        .await
        .map_err(|e| Error::DeploymentError(format!("Transaction failed: {}", e)))?
        .ok_or_else(|| Error::DeploymentError("Transaction receipt not found".to_string()))?;

    Ok(receipt)
}

fn print_deployment_info(
    receipt: &ethers::core::types::TransactionReceipt,
    network: &NetworkConfig,
) {
    println!("\n✅ Contract deployed successfully\n{}", network);
    if let Some(contract_address) = receipt.contract_address {
        println!("📍 Contract address: {:?}", contract_address);
    }
    println!("🧾 Transaction hash: {:?}", receipt.transaction_hash);

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

#[derive(Debug, Clone)]
struct NetworkConfig {
    name: String,
    endpoint: String,
    chain_id: u64,
}

impl NetworkConfig {
    /// Create a NetworkConfig based on DeployArgs
    fn from_args(args: &DeployArgs) -> Result<Self, Error> {
        if args.local {
            Ok(NetworkConfig {
                name: "local".to_string(),
                endpoint: "http://localhost:8545".to_string(),
                chain_id: 1337,
            })
        } else if args.dev {
            Ok(NetworkConfig {
                name: "dev".to_string(),
                endpoint: "https://rpc.dev.gblend.xyz".to_string(),
                chain_id: 20993,
            })
        } else if let (Some(rpc), Some(chain_id)) = (&args.rpc, args.chain_id) {
            Ok(NetworkConfig {
                name: "Custom".to_string(),
                endpoint: rpc.clone(),
                chain_id,
            })
        } else {
            Err(Error::NetworkError(
                "Please specify either --local, --dev, or both --rpc and --chain-id.".to_string(),
            ))
        }
    }
}

impl fmt::Display for NetworkConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Network: {}\nEndpoint: {}\nChain ID: {}",
            self.name, self.endpoint, self.chain_id
        )
    }
}
