mod cli;
mod commands;
mod config;
mod error;
mod utils;

use cli::Cli;

#[tokio::main]
async fn main() {
    if let Err(e) = Cli::new().unwrap().execute().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
