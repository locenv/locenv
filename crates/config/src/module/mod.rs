use config_macros::Config;
use serde::Deserialize;

#[derive(Config, Deserialize)]
pub struct ModuleDefinition {
    pub name: String,
    pub version: u32,
    pub program: Program,
}

#[derive(Deserialize)]
pub enum Program {
    Script(String),
    Binary(BinaryProgram),
}

#[derive(Deserialize)]
pub struct BinaryProgram {
    pub linux: Option<String>,
    pub darwin: Option<String>,
    pub win32: Option<String>,
}

impl BinaryProgram {
    #[cfg(target_os = "linux")]
    pub fn current(&self) -> Option<&str> {
        self.linux.as_deref()
    }

    #[cfg(target_os = "macos")]
    pub fn current(&self) -> Option<&PlatformConfigurations> {
        self.darwin.as_deref()
    }

    #[cfg(target_os = "windows")]
    pub fn current(&self) -> Option<&PlatformConfigurations> {
        self.win32.as_deref()
    }
}
