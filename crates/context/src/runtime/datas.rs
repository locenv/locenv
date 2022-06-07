use super::Runtime;
use std::path::PathBuf;

pub struct Datas<'context> {
    runtime: Runtime<'context>,
    name: &'static str,
}

impl<'context> Datas<'context> {
    pub(super) fn new(runtime: Runtime<'context>, name: &'static str) -> Self {
        Datas { runtime, name }
    }

    pub fn path(&self) -> PathBuf {
        self.runtime.path().join(self.name)
    }
}
