use super::Datas;
use fmap_macros::Directory;
use std::borrow::Cow;
use std::marker::PhantomData;
use std::path::PathBuf;

/// Represents where to store modules.
pub struct Modules<'context> {
    parent: Datas<'context>,
    name: &'static str,
}

impl<'context> Modules<'context> {
    pub(super) fn new(parent: Datas<'context>, name: &'static str) -> Self {
        Self { parent, name }
    }

    pub fn by_name<'name>(self, name: Cow<'name, str>) -> Module<'context, 'name> {
        Module::new(self, name)
    }

    pub fn path(&self) -> PathBuf {
        let mut path = self.parent.path();
        path.push(self.name);
        path
    }
}

/// Represents a directory that contain a module.
pub trait ModuleDirectory {
    fn path(&self) -> PathBuf;

    fn definition(&self) -> PathBuf {
        let mut path = self.path();
        path.push("locenv-module.yml");
        path
    }
}

/// Represents location of a module.
#[derive(Directory)]
pub struct Module<'context, 'name> {
    parent: Modules<'context>,
    name: Cow<'name, str>,

    #[directory(name = ".locenv", pub)]
    metadata: PhantomData<ModuleMetadata<'context, 'name>>,
}

impl<'context, 'name> Module<'context, 'name> {
    fn new(parent: Modules<'context>, name: Cow<'name, str>) -> Self {
        Self {
            parent,
            name,
            metadata: PhantomData,
        }
    }

    pub fn path(&self) -> PathBuf {
        let mut path = self.parent.path();
        path.push(self.name.as_ref());
        path
    }
}

impl<'context, 'name> ModuleDirectory for Module<'context, 'name> {
    fn path(&self) -> PathBuf {
        let mut path = self.parent.path();
        path.push(self.name.as_ref());
        path
    }
}

/// Represents the location of module metadata.
pub struct ModuleMetadata<'context, 'module> {
    parent: Module<'context, 'module>,
    name: &'static str,
}

impl<'context, 'module> ModuleMetadata<'context, 'module> {
    fn new(parent: Module<'context, 'module>, name: &'static str) -> Self {
        Self { parent, name }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn path(&self) -> PathBuf {
        let mut path = self.parent.path();
        path.push(self.name);
        path
    }
}
