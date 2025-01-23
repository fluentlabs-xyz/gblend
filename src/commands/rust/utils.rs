use crate::error::Error;
use std::process::Command;

pub(crate) enum Tool {
    Cargo,
    Rustup,
    WasmTarget,
    Wasm2Wat,
}

impl Tool {
    // Create a list of all required tools, conditionally including `wasm2wat`
    pub fn all(include_wasm2wat: bool) -> Vec<Self> {
        let mut deps = vec![Self::Cargo, Self::Rustup, Self::WasmTarget];
        if include_wasm2wat {
            deps.push(Self::Wasm2Wat);
        }
        deps
    }

    // Ensure the dependency is installed or attempt to install it
    pub fn ensure(&self) -> Result<(), Error> {
        if !self.is_installed() {
            println!("🔍 {} not found. Attempting to install...", self);
            self.install()?;
        }
        Ok(())
    }

    // Check if the dependency is installed
    pub fn is_installed(&self) -> bool {
        match self {
            Self::Cargo | Self::Rustup => Command::new(self.command())
                .arg("--version")
                .output()
                .is_ok(),
            Self::WasmTarget => Command::new("rustup")
                .args(["target", "list", "--installed"])
                .output()
                .map_or(false, |output| {
                    String::from_utf8_lossy(&output.stdout).contains("wasm32-unknown-unknown")
                }),
            Self::Wasm2Wat => Command::new(self.command())
                .arg("--version")
                .output()
                .is_ok(),
        }
    }

    // Attempt to install the dependency, if possible
    pub fn install(&self) -> Result<(), Error> {
        match self {
            Self::Cargo => Err(Error::Build(
                "Cargo is not installed. Please install Rust and Cargo from https://rustup.rs/.".to_string(),
            )),
            Self::Rustup => Err(Error::Build(
                "Rustup is not installed. Please install Rustup from https://rustup.rs/.".to_string(),
            )),
            Self::WasmTarget => {
                println!("Adding wasm32-unknown-unknown target via rustup...");
                Command::new("rustup")
                    .args(["target", "add", "wasm32-unknown-unknown"])
                    .status()
                    .map_err(|_| Error::Build("Failed to add wasm32-unknown-unknown target.".to_string()))
                    .and_then(|status| {
                        if status.success() {
                            println!("✅ Successfully added wasm32-unknown-unknown target.");
                            Ok(())
                        } else {
                            Err(Error::Build(
                                "Failed to add wasm32-unknown-unknown target.".to_string(),
                            ))
                        }
                    })
            }
            Self::Wasm2Wat => Err(Error::Build(
                "wasm2wat is not installed. Please install it:\n- For MacOS: `brew install wabt`\n- For Linux: check your package manager\n- For Windows: download from https://github.com/WebAssembly/wabt/releases".to_string(),
            )),
        }
    }

    // Get the command name associated with each dependency
    pub fn command(&self) -> &str {
        match self {
            Self::Cargo => "cargo",
            Self::Rustup => "rustup",
            Self::WasmTarget => "rustup",
            Self::Wasm2Wat => "wasm2wat",
        }
    }
}

// Implement Display for Dependency for user-friendly messages
impl std::fmt::Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cargo => write!(f, "Cargo"),
            Self::Rustup => write!(f, "Rustup"),
            Self::WasmTarget => write!(f, "wasm32-unknown-unknown target"),
            Self::Wasm2Wat => write!(f, "wasm2wat"),
        }
    }
}
