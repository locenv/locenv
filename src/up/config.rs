use serde::{Serialize, Deserialize};
use std::fs::File;

#[derive(Deserialize, Serialize)]
pub struct Services {
}

impl Services {
    pub fn load<P: AsRef<str>>(path: P) -> Result<Self, String> {
        let p = path.as_ref();
        let f = File::open(p).or_else(|e| { Err(format!("Failed to open {p}: {e}")) })?;

        serde_yaml::from_reader(f).or_else(|e| { Err(format!("{p}: {e}")) })
    }
}
