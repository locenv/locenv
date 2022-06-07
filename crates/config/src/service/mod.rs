use crate::FromFileError;
use config_macros::Config;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Config, Deserialize)]
pub struct ServiceDefinition {
    pub linux: Option<PlatformConfigurations>,
    pub darwin: Option<PlatformConfigurations>,
    pub win32: Option<PlatformConfigurations>,
}

#[derive(Clone, Deserialize)]
pub struct PlatformConfigurations {
    pub build: Option<BuildConfigurations>,
}

#[derive(Clone, Deserialize)]
pub struct BuildConfigurations {
    pub steps: Vec<BuildStep>,
}

#[derive(Clone, Deserialize)]
pub struct BuildStep {
    pub uses: String,
    pub name: Option<String>,
    pub with: Option<HashMap<String, String>>,
}

// ServiceConfigurations

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

// PlatformConfigurations

impl PlatformConfigurations {
    pub fn from_service_definition_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<Option<Self>, FromFileError> {
        let def = ServiceDefinition::from_file(path)?;

        Ok(def.flatten())
    }
}
