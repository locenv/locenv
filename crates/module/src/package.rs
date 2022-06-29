use context::modules::module::ModuleContent;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::remove_dir_all;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Clone, Copy)]
pub enum Registry {
    GitHub,
}

pub enum RegistryError {
    Invalid,
}

#[derive(Clone)]
pub struct PackageId {
    registry: Registry,
    name: String,
}

#[derive(Debug)]
pub enum IdentifierError {
    InvalidFormat,
    UnknowRegistry,
}

pub struct PackageReader<'content> {
    content: &'content Path,
}

pub struct InstallationScope<'destination> {
    destination: &'destination Path,
    succeeded: bool,
}

// Registry

impl Display for Registry {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Registry::GitHub => f.write_str("github"),
        }
    }
}

impl FromStr for Registry {
    type Err = RegistryError;

    fn from_str(s: &str) -> Result<Self, RegistryError> {
        let v = match s.to_lowercase().as_str() {
            "github" => Registry::GitHub,
            _ => return Err(RegistryError::Invalid),
        };

        Ok(v)
    }
}

// PackageId

impl PackageId {
    pub fn registry(&self) -> Registry {
        self.registry
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl Display for PackageId {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.registry, self.name)
    }
}

impl FromStr for PackageId {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, IdentifierError> {
        // Parse.
        let (registry, name) = match s.split_once(':') {
            Some(v) => v,
            None => return Err(IdentifierError::InvalidFormat),
        };

        if registry.is_empty() || name.is_empty() {
            return Err(IdentifierError::InvalidFormat);
        }

        // Parse registry.
        let registry: Registry = match registry.parse() {
            Ok(r) => r,
            Err(_) => return Err(IdentifierError::UnknowRegistry),
        };

        Ok(PackageId {
            registry,
            name: name.into(),
        })
    }
}

// IdentifierError

impl Error for IdentifierError {}

impl Display for IdentifierError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            IdentifierError::InvalidFormat => f.write_str("Unrecognized format"),
            IdentifierError::UnknowRegistry => f.write_str("Unrecognized registry"),
        }
    }
}

// PackageReader

impl<'content> PackageReader<'content> {
    pub fn new(content: &'content Path) -> Self {
        PackageReader { content }
    }
}

impl<'content> ModuleContent for PackageReader<'content> {
    fn path(&self) -> PathBuf {
        self.content.into()
    }
}

// InstallationScope

impl<'destination> InstallationScope<'destination> {
    pub fn new(destination: &'destination Path) -> Self {
        InstallationScope {
            destination,
            succeeded: false,
        }
    }

    pub fn success(&mut self) {
        self.succeeded = true;
    }
}

impl<'destination> Drop for InstallationScope<'destination> {
    fn drop(&mut self) {
        if !self.succeeded {
            remove_dir_all(self.destination).unwrap();
        }
    }
}
