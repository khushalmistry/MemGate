//! Error types

use std::fmt;

/// MemGate error type
#[derive(Debug)]
pub enum Error {
    /// IO error
    Io(std::io::Error),
    /// Memory access error
    MemoryAccess(String),
    /// Process error
    Process(String),
    /// Invalid address
    InvalidAddress(u64),
    /// Permission denied
    PermissionDenied(String),
    /// Generic error
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::MemoryAccess(msg) => write!(f, "Memory access error: {}", msg),
            Error::Process(msg) => write!(f, "Process error: {}", msg),
            Error::InvalidAddress(addr) => write!(f, "Invalid address: {:016x}", addr),
            Error::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            Error::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Error::Other(e)
    }
}

impl From<&str> for Error {
    fn from(e: &str) -> Self {
        Error::Other(e.to_string())
    }
}

/// Result type for MemGate operations
pub type Result<T> = std::result::Result<T, Error>;