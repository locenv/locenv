use super::Modules;
use std::borrow::Cow;
use std::path::PathBuf;

pub struct Module<'context, 'name> {
    parent: Modules<'context>,
    name: Cow<'name, str>,
}

impl<'context, 'name> Module<'context, 'name> {
    pub(super) fn new(parent: Modules<'context>, name: Cow<'name, str>) -> Self {
        Module { parent, name }
    }

    pub fn definition(&self) -> PathBuf {
        let mut p = self.path();
        p.push("locenv-module.yml");
        p
    }

    pub fn path(&self) -> PathBuf {
        let mut p = self.parent.path();
        p.push(self.name.as_ref());
        p
    }
}
