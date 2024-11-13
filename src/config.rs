use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(next_help_heading = "Environment configuration")]
pub struct EnvConfig {
    #[arg(long, global = true, help = "Path to .env file")]
    env_file: Option<PathBuf>,

    #[arg(long, global = true, help = "Environment name to load .env.<env>")]
    env: Option<String>,
}

impl EnvConfig {
    pub fn load() -> Result<Self, dotenvy::Error> {
        let env_args: Vec<String> = std::env::args()
            .filter(|arg| arg.starts_with("--env"))
            .collect();

        let config = Self::parse_from(env_args);
        config.init()?;

        Ok(config)
    }

    fn init(&self) -> Result<(), dotenvy::Error> {
        if let Some(path) = &self.env_file {
            dotenvy::from_path(path)?;
            return Ok(());
        }

        if let Some(env) = &self.env {
            dotenvy::from_filename(format!(".env.{}", env))?;
            return Ok(());
        }

        match dotenvy::dotenv() {
            Ok(_) => Ok(()),
            // Ignore error if .env file is not found
            Err(e) if e.not_found() => Ok(()),
            Err(e) => Err(e),
        }
    }
}
