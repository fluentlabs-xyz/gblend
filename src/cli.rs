use crate::{
    commands::{
        legacy_init::legacy_init,
        rust::{self, RustCommand},
    },
    config::Config,
    error::Error,
};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// CLI tool to scaffold, build, and deploy contracts on Fluent
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[arg(long, global = true, help = "Path to .env file")]
    env_file: Option<PathBuf>,

    #[arg(long, global = true, help = "Environment name to load .env.<env>")]
    env: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new project
    Init(InitCommand),
    /// Build the project
    Build(BuildCommand),
    /// Deploy the compiled WASM file to a specified network
    Deploy(DeployCommand),
}

#[derive(Args)]
pub struct InitCommand {
    #[command(subcommand)]
    pub mode: Option<InitMode>,
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
    /// Arguments for deploying the project
    #[command(flatten)]
    pub args: rust::DeployArgs,
}

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }

    pub async fn execute(&self) -> Result<(), Error> {
        Config::new(self.env_file.clone(), self.env.clone()).load()?;

        match &self.command {
            Commands::Init(cmd) => match &cmd.mode {
                Some(InitMode::Rust(args)) => RustCommand::init(args),
                None => legacy_init()
                    .await
                    .map_err(|e| Error::InitializationError(e.to_string())),
            },
            Commands::Build(cmd) => match &cmd.mode {
                BuildMode::Rust(args) => RustCommand::build(args),
            },
            Commands::Deploy(cmd) => RustCommand::deploy(&cmd.args).await,
        }
    }
}
