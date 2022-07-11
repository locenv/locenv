use serde::Deserialize;

#[derive(Deserialize)]
pub struct ModuleDefinition {
    pub name: String,
    pub version: u32,
    pub program: Program,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Program {
    Script(String),
    Binary(BinaryProgram),
}

/// Represents native programs for each platform.
#[derive(Deserialize)]
pub struct BinaryProgram {
    pub linux: Option<BinaryFiles>,
    pub darwin: Option<BinaryFiles>,
    pub win32: Option<BinaryFiles>,
}

impl BinaryProgram {
    #[cfg(target_os = "linux")]
    pub fn current(&self) -> Option<&BinaryFiles> {
        self.linux.as_ref()
    }

    #[cfg(target_os = "macos")]
    pub fn current(&self) -> Option<&BinaryFiles> {
        self.darwin.as_ref()
    }

    #[cfg(target_os = "windows")]
    pub fn current(&self) -> Option<&BinaryFiles> {
        self.win32.as_ref()
    }
}

/// Represents native programs for each CPU type.
#[derive(Deserialize)]
pub struct BinaryFiles {
    pub aarch64: Option<String>,
    pub amd64: Option<String>,
}

impl BinaryFiles {
    #[cfg(target_arch = "aarch64")]
    pub fn current(&self) -> Option<&str> {
        self.aarch64.as_deref()
    }

    #[cfg(target_arch = "x86_64")]
    pub fn current(&self) -> Option<&str> {
        self.amd64.as_deref()
    }
}
