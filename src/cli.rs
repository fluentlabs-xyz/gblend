use crate::{
    commands::rust::{self, RustCommand},
    error::Error,
};
use clap::{Args, Parser, Subcommand};

/// CLI tool to scaffold, build, and deploy contracts on Fluent
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new project
    Init(InitCommand),
    /// Build the project
    Build(BuildCommand),
    /// Deploy the compiled WASM file to specified network
    Deploy(DeployCommand),
}

#[derive(Args)]
pub struct InitCommand {
    #[command(subcommand)]
    pub mode: InitMode,
}

#[derive(Subcommand)]
pub enum InitMode {
    /// Initialize Rust smart contract project
    Rust(rust::InitArgs),
}

#[derive(Args)]
pub struct BuildCommand {
    #[command(subcommand)]
    pub mode: BuildMode,
}

#[derive(Subcommand)]
pub enum BuildMode {
    /// Build Rust smart contract project
    Rust(rust::BuildArgs),
}

#[derive(Args)]
pub struct DeployCommand {
    #[command(subcommand)]
    pub network: NetworkType,
}

#[derive(Subcommand)]
pub enum NetworkType {
    /// Deploy to local development network
    Local(rust::DeployArgs),
    /// Deploy to development testnet
    Dev(rust::DeployArgs),
}

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }

    pub async fn execute(&self) -> Result<(), Error> {
        match &self.command {
            Commands::Init(cmd) => match &cmd.mode {
                InitMode::Rust(args) => RustCommand::init(args),
            },
            Commands::Build(cmd) => match &cmd.mode {
                BuildMode::Rust(args) => RustCommand::build(args),
            },
            Commands::Deploy(cmd) => match &cmd.network {
                NetworkType::Local(args) => RustCommand::deploy(args, "local").await,
                NetworkType::Dev(args) => RustCommand::deploy(args, "dev").await,
            },
        }
    }
}
