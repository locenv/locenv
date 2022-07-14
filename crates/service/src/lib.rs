use serde::de::Visitor;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Write};
use url::Url;

pub mod repository;

#[derive(Deserialize)]
pub struct ApplicationConfiguration {
    pub configurations: HashMap<String, ServiceConfigurations>,
    pub instances: HashMap<String, InstanceConfigurations>,
}

#[derive(Deserialize)]
pub struct ServiceConfigurations {
    pub repository: RepositoryConfigurations,
}

#[derive(Deserialize)]
pub struct InstanceConfigurations {
    pub configuration: String,
}

#[derive(Deserialize)]
pub struct RepositoryConfigurations {
    pub uri: RepositoryUri,
    pub r#type: RepositoryType,

    #[serde(flatten)]
    pub options: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RepositoryUri {
    Scp(ScpUrl),
    Url(Url),
}

/// Represents a URL for Secure copy protocol (SCP).
#[derive(Debug)]
pub struct ScpUrl {
    pub user: Option<String>,
    pub host: String,
    pub path: Option<String>,
}

impl Display for ScpUrl {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if let Some(user) = &self.user {
            f.write_str(user)?;
            f.write_char('@')?;
        }

        f.write_str(&self.host)?;

        if let Some(path) = &self.path {
            f.write_char(':')?;
            f.write_str(path)?;
        }

        Ok(())
    }
}

impl<'de> Deserialize<'de> for ScpUrl {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(ScpUrlVisitor)
    }
}

struct ScpUrlVisitor;

impl<'de> Visitor<'de> for ScpUrlVisitor {
    type Value = ScpUrl;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "a string in SCP-syntax")
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<ScpUrl, E> {
        // Extract components.
        let mut buffer = String::with_capacity(v.len());
        let mut user: Option<String> = None;
        let mut host: Option<String> = None;

        for c in v.chars() {
            if host.is_none() {
                if c == '@' {
                    if user.is_some() || buffer.is_empty() {
                        return Err(E::custom("SCP-syntax is not valid"));
                    }

                    user = Some(buffer.clone());
                    buffer.truncate(0);
                } else if c == ':' {
                    if buffer.is_empty() {
                        return Err(E::custom("SCP-syntax is not valid"));
                    }

                    host = Some(buffer.clone());
                    buffer.truncate(0);
                } else {
                    buffer.push(c);
                }
            } else if c == '/' && buffer.is_empty() {
                return Err(E::custom("SCP-syntax is not valid"));
            } else {
                buffer.push(c);
            }
        }

        // Construct result.
        if let Some(host) = host {
            let path = if buffer.is_empty() {
                None
            } else {
                buffer.shrink_to_fit();
                Some(buffer)
            };

            Ok(ScpUrl { user, host, path })
        } else if user.is_none() && !buffer.is_empty() {
            buffer.shrink_to_fit();

            Ok(ScpUrl {
                user: None,
                host: buffer,
                path: None,
            })
        } else {
            Err(E::custom("SCP-syntax is not valid"))
        }
    }
}

/// Type of repository.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryType {
    Git,
}

/// Represents a set of configuration to define how to interact with the service like how to build, etc.
#[derive(Deserialize)]
pub struct ServiceDefinition {
    pub linux: Option<PlatformConfigurations>,
    pub darwin: Option<PlatformConfigurations>,
    pub win32: Option<PlatformConfigurations>,
}

impl ServiceDefinition {
    pub fn flatten(&self) -> Option<PlatformConfigurations> {
        if let Some(v) = self.current() {
            Some(v.clone())
        } else {
            None
        }
    }

    #[cfg(target_os = "linux")]
    pub fn current(&self) -> Option<&PlatformConfigurations> {
        self.linux.as_ref()
    }

    #[cfg(target_os = "macos")]
    pub fn current(&self) -> Option<&PlatformConfigurations> {
        self.darwin.as_ref()
    }

    #[cfg(target_os = "windows")]
    pub fn current(&self) -> Option<&PlatformConfigurations> {
        self.win32.as_ref()
    }
}

/// Represents a platform-specific configuration for a service.
#[derive(Clone, Deserialize)]
pub struct PlatformConfigurations {
    pub build: Option<String>,
}
