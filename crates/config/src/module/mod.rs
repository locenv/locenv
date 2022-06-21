use config_macros::Config;
use serde::Deserialize;

#[derive(Config, Deserialize)]
pub struct ModuleDefinition {
    pub name: String,
    pub file: String,
}
