use context::Context;
use std::path::{Path, PathBuf};

pub struct Module {
    path: PathBuf,
}

impl Module {
    pub fn find(context: &Context, name: &str) -> Option<Self> {
        let path = context.modules().by_name(name).path();

        if !path.is_dir() {
            return None;
        }

        Some(Module { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
