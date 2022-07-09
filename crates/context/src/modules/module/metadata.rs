use super::{Module, ModuleContent};
use std::path::PathBuf;

pub struct Metadata<'context, 'module> {
    parent: Module<'context, 'module>,
    name: &'static str,
}

impl<'context, 'module> Metadata<'context, 'module> {
    pub(super) fn new(parent: Module<'context, 'module>, name: &'static str) -> Self {
        Metadata { parent, name }
    }

    pub fn path(&self) -> PathBuf {
        let mut p = self.parent.path();
        p.push(self.name);
        p
    }

    pub fn name(&self) -> &str {
        self.name
    }
}
