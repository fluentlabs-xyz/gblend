use std::{fmt, io};

#[derive(Debug)]
pub enum Error {
    /// IO operation error
    Io(io::Error),
    /// Project initialization error
    InitializationError(String),
    ConfigError(String),
    /// Build process error
    BuildError(String),
    /// Deployment error
    DeploymentError(String),
    /// Network error
    NetworkError(String),
    /// Invalid project structure
    InvalidProject(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::InitializationError(msg) => write!(f, "Initialization error: {}", msg),
            Error::ConfigError(msg) => write!(f, "Config error: {}", msg),
            Error::BuildError(msg) => write!(f, "Build error: {}", msg),
            Error::DeploymentError(msg) => write!(f, "Deployment error: {}", msg),
            Error::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Error::InvalidProject(msg) => write!(f, "Invalid project: {}", msg),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}
