use crate::error::Error;
use std::{path::PathBuf, process::Command};
use tempfile::TempDir;

pub struct Repository {
    _temp_dir: TempDir, // Using underscore to indicate this field keeps the TempDir alive
    repo_path: PathBuf,
}

impl Repository {
    pub fn clone_fluentbase() -> Result<Self, Error> {
        println!("📦 Cloning Fluentbase repository...");

        // Create temporary directory
        let temp_dir = TempDir::new().map_err(|e| {
            Error::InitializationError(format!("Failed to create temporary directory: {}", e))
        })?;

        let repo_path = temp_dir.path().to_path_buf();

        // Clone repository
        let output = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "--branch",
                "devel",
                "https://github.com/fluentlabs-xyz/fluentbase.git",
                repo_path.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| {
                Error::InitializationError(format!("Failed to clone repository: {}", e))
            })?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(Error::InitializationError(format!(
                "Failed to clone repository: {}",
                error
            )));
        }

        Ok(Self {
            _temp_dir: temp_dir,
            repo_path,
        })
    }

    pub fn get_examples_path(&self) -> PathBuf {
        self.repo_path.join("examples")
    }

    pub fn get_example_path(&self, example_name: &str) -> PathBuf {
        self.get_examples_path().join(example_name)
    }
}
