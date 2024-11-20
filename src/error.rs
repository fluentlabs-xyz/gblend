use std::{fmt, io};

#[derive(Debug)]
pub enum Error {
    /// IO operation error
    Io(io::Error),
    /// Project initialization error
    Initialization(String),
    Config(String),
    /// Build process error
    Build(String),
    /// Deployment error
    Deployment(String),
    /// Network error
    Network(String),
    /// Invalid project structure
    InvalidProject(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Initialization(msg) => write!(f, "Initialization error: {}", msg),
            Error::Config(msg) => write!(f, "Config error: {}", msg),
            Error::Build(msg) => write!(f, "Build error: {}", msg),
            Error::Deployment(msg) => write!(f, "Deployment error: {}", msg),
            Error::Network(msg) => write!(f, "Network error: {}", msg),
            Error::InvalidProject(msg) => write!(f, "Invalid project: {}", msg),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}
