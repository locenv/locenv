use std::path::{Path, PathBuf};

pub struct Context {
    project: PathBuf,
    runtime: PathBuf,
}

impl Context {
    pub fn new<P: Into<PathBuf>>(project: P) -> Self {
        let owned = project.into();

        Context {
            runtime: owned.join(".locenv"),
            project: owned,
        }
    }

    pub fn project_dir(&self) -> &Path {
        &self.project
    }

    pub fn runtime_dir(&self) -> &Path {
        &self.runtime
    }

    pub fn repositories_dir(&self) -> PathBuf {
        self.runtime.join("repositories")
    }

    pub fn manager_dir(&self) -> PathBuf {
        self.runtime.join("manager")
    }

    pub fn services_config(&self) -> PathBuf {
        self.project_dir().join("locenv-services.yml")
    }

    pub fn repository_dir<N: AsRef<Path>>(&self, name: N) -> PathBuf {
        let mut r = self.repositories_dir();
        r.push(name);
        r
    }
}
