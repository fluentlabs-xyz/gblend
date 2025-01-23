mod cli;
mod commands;
mod config;
mod error;
mod utils;

use cli::Cli;

#[tokio::main]
#[allow(clippy::needless_return)]
async fn main() {
    Cli::new().unwrap().execute().await.unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1)
    });
}
