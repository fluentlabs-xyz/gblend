use crate::error::Error;
use std::path::PathBuf;

/// Configuration loader for environment variables
pub struct Config {
    env: Option<String>,
    env_file: Option<PathBuf>,
}

impl Config {
    pub fn new(env_file: Option<PathBuf>, env: Option<String>) -> Self {
        Self { env_file, env }
    }

    pub fn load(&self) -> Result<(), Error> {
        if let Err(e) = dotenvy::from_filename(".env") {
            if e.not_found() {
                // No .env file found, continue
            } else {
                return Err(Error::ConfigError(format!("Failed to load .env: {e}")));
            }
        }

        if let Some(path) = &self.env_file {
            dotenvy::from_path(path).map_err(|e| {
                Error::ConfigError(format!(
                    "Failed to load specified env file {}: {}",
                    path.display(),
                    e
                ))
            })?;
        }

        if let Some(env) = &self.env {
            self.load_env_file(env)?;
        }

        Ok(())
    }

    fn load_env_file(&self, env: &str) -> Result<(), Error> {
        let filename = format!(".env.{env}");
        dotenvy::from_filename(&filename)
            .map_err(|e| Error::ConfigError(format!("Failed to load {filename}: {e}")))?;
        Ok(())
    }
}
