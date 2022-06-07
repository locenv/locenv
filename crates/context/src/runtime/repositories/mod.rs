use self::repository::Repository;
use super::Runtime;
use std::borrow::Cow;
use std::path::PathBuf;

pub mod repository;

pub struct Repositories<'context> {
    runtime: Runtime<'context>,
    name: &'static str,
}

impl<'context> Repositories<'context> {
    pub(super) fn new(runtime: Runtime<'context>, name: &'static str) -> Self {
        Repositories { runtime, name }
    }

    pub fn by_name<'name>(self, name: &'name str) -> Repository<'context, 'name> {
        Repository::new(self, Cow::Borrowed(name))
    }

    pub fn path(&self) -> PathBuf {
        self.runtime.path().join(self.name)
    }
}
