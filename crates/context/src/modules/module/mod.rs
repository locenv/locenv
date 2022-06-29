use self::metadata::Metadata;
use super::Modules;
use std::borrow::Cow;
use std::path::PathBuf;

pub mod metadata;

pub struct Module<'context, 'name> {
    parent: Modules<'context>,
    name: Cow<'name, str>,
}

pub trait ModuleContent {
    fn path(&self) -> PathBuf;

    fn definition(&self) -> PathBuf {
        let mut path = self.path();
        path.push("locenv-module.yml");
        path
    }
}

// Module

impl<'context, 'name> Module<'context, 'name> {
    pub(super) fn new(parent: Modules<'context>, name: Cow<'name, str>) -> Self {
        Module { parent, name }
    }

    pub fn metadata(self) -> Metadata<'context, 'name> {
        Metadata::new(self, ".locenv")
    }
}

impl<'context, 'name> ModuleContent for Module<'context, 'name> {
    fn path(&self) -> PathBuf {
        let mut path = self.parent.path();
        path.push(self.name.as_ref());
        path
    }
}
