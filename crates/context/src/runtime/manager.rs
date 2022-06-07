use super::Runtime;
use std::path::PathBuf;

pub struct Manager<'context> {
    runtime: Runtime<'context>,
    name: &'static str,
}

impl<'context> Manager<'context> {
    pub(super) fn new(runtime: Runtime<'context>, name: &'static str) -> Self {
        Manager { runtime, name }
    }

    pub fn port(&self) -> PathBuf {
        let mut p = self.path();
        p.push("port");
        p
    }

    pub fn path(&self) -> PathBuf {
        self.runtime.path().join(self.name)
    }
}
