use std::path::{Path, PathBuf};

pub struct Project<'context> {
    path: &'context Path,
}

impl<'context> Project<'context> {
    pub(crate) fn new(path: &'context Path) -> Self {
        Project { path }
    }

    pub fn services_config(&self) -> PathBuf {
        self.path.join("locenv-services.yml")
    }

    pub fn path(&self) -> &Path {
        self.path
    }
}
