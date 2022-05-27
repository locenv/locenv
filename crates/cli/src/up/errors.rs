use std::error::Error;
use std::fmt::{Display, Formatter, Result};
use std::path::PathBuf;

#[derive(Debug)]
pub struct ConfigOpenError {
    path: PathBuf,
    reason: std::io::Error,
}

#[derive(Debug)]
pub struct ConfigParseError {
    path: PathBuf,
    reason: serde_yaml::Error,
}

#[derive(Debug)]
pub struct ServiceManagerPortFileOpenError {
    path: PathBuf,
    reason: std::io::Error,
}

#[derive(Debug)]
pub struct AlreadyRunning;

// ConfigOpenError

impl ConfigOpenError {
    pub fn new<P: Into<PathBuf>>(path: P, reason: std::io::Error) -> Self {
        ConfigOpenError {
            path: path.into(),
            reason,
        }
    }
}

impl Error for ConfigOpenError {}

impl Display for ConfigOpenError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Failed to open {}: {}", self.path.display(), self.reason)
    }
}

// ConfigParseError

impl ConfigParseError {
    pub fn new<P: Into<PathBuf>>(path: P, reason: serde_yaml::Error) -> Self {
        ConfigParseError {
            path: path.into(),
            reason,
        }
    }
}

impl Error for ConfigParseError {}

impl Display for ConfigParseError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "Failed to parse {}: {}",
            self.path.display(),
            self.reason
        )
    }
}

// ServiceManagerPortFileOpenError

impl ServiceManagerPortFileOpenError {
    pub fn new<P: Into<PathBuf>>(path: P, reason: std::io::Error) -> Self {
        ServiceManagerPortFileOpenError {
            path: path.into(),
            reason,
        }
    }
}

impl Error for ServiceManagerPortFileOpenError {}

impl Display for ServiceManagerPortFileOpenError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Failed to open {}: {}", self.path.display(), self.reason)
    }
}

// AlreadyRunning

impl AlreadyRunning {
    pub fn new() -> Self {
        AlreadyRunning {}
    }
}

impl Error for AlreadyRunning {}

impl Display for AlreadyRunning {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "The services already running")
    }
}
