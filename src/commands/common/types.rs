use crate::error::Error;
use std::path::PathBuf;

/// Result of the build process
#[derive(Debug)]
pub struct BuildResult {
    /// Path to the generated WASM file
    pub wasm_path: PathBuf,
    /// Size of the generated WASM file in bytes
    pub size: u64,
    /// Optional warnings from the build process
    pub warnings: Option<Vec<String>>,
    /// Optional metadata about the build
    pub metadata: Option<BuildMetadata>,
}

/// Additional metadata about the build
#[derive(Debug)]
pub struct BuildMetadata {
    /// Time taken to build
    pub build_time: std::time::Duration,
    /// Compiler version used
    pub compiler_version: String,
    /// Target architecture
    pub target: String,
    /// Optimization level
    pub optimization_level: String,
}

/// Common trait for all project builders
pub trait ProjectBuilder {
    /// Initialize a new project
    fn init() -> Result<(), Error>;

    /// Build project from the specified path
    fn build(path: &PathBuf) -> Result<BuildResult, Error>;

    /// Validate project structure
    fn validate_project(path: &PathBuf) -> Result<(), Error>;

    /// Get project template path
    fn template_path() -> PathBuf;
}

/// Network configuration for deployment
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Network endpoint URL
    pub endpoint: String,
    /// Chain ID
    pub chain_id: u64,
    /// Network type (local, testnet, mainnet)
    pub network_type: NetworkType,
}

/// Type of network for deployment
#[derive(Debug, Clone)]
pub enum NetworkType {
    /// Local development network
    Local,
    /// Development testnet
    Dev,
}
