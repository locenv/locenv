use std::path::{PathBuf, Path};

pub struct Context {
    project: PathBuf,
    path: PathBuf
}

impl Context {
    pub fn new(project: PathBuf) -> Self {
        Context { path: project.join(".locenv"), project }
    }

    pub fn project(&self) -> &Path {
        &self.project
    }

    pub fn repositories(&self) -> PathBuf {
        self.path.join("repositories")
    }

    pub fn service_manager(&self) -> PathBuf {
        self.path.join("service-manager")
    }
}
