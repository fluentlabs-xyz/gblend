use std::{fmt, io, path::PathBuf};

#[derive(Debug)]
pub enum Error {
    /// IO operation error
    Io(io::Error),
    /// Project initialization error
    InitializationError(String),
    /// Build process error
    BuildError(String),
    /// Deployment error
    DeploymentError(String),
    /// Network error
    NetworkError(String),
    /// Network not specified
    NetworkNotSpecified,
    /// Invalid project structure
    InvalidProject(String),
    /// Invalid path
    InvalidPath(PathBuf),
    /// Invalid private key
    InvalidPrivateKey(String),
    /// WASM validation error
    WasmValidationError(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::InitializationError(msg) => write!(f, "Initialization error: {}", msg),
            Error::BuildError(msg) => write!(f, "Build error: {}", msg),
            Error::DeploymentError(msg) => write!(f, "Deployment error: {}", msg),
            Error::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Error::NetworkNotSpecified => write!(f, "Network not specified. Use --local or --dev"),
            Error::InvalidProject(msg) => write!(f, "Invalid project: {}", msg),
            Error::InvalidPath(path) => write!(f, "Invalid path: {}", path.display()),
            Error::InvalidPrivateKey(msg) => write!(f, "Invalid private key: {}", msg),
            Error::WasmValidationError(msg) => write!(f, "WASM validation error: {}", msg),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}
