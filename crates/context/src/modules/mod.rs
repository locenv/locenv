use self::module::Module;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

pub mod module;

pub struct Modules<'context> {
    prefix: &'context Path,
    name: &'static str,
}

impl<'context> Modules<'context> {
    pub(super) fn new(prefix: &'context Path, name: &'static str) -> Self {
        Modules { prefix, name }
    }

    pub fn by_name<'name>(self, name: &'name str) -> Module<'context, 'name> {
        Module::new(self, Cow::Borrowed(name))
    }

    pub fn by_owned<'name, N: Into<String>>(self, name: N) -> Module<'context, 'name> {
        Module::new(self, Cow::Owned(name.into()))
    }

    pub fn path(&self) -> PathBuf {
        self.prefix.join(self.name)
    }
}
