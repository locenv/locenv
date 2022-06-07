use super::Repositories;
use std::borrow::Cow;
use std::path::PathBuf;

pub struct Repository<'context, 'name> {
    parent: Repositories<'context>,
    name: Cow<'name, str>,
}

impl<'context, 'name> Repository<'context, 'name> {
    pub(super) fn new(parent: Repositories<'context>, name: Cow<'name, str>) -> Self {
        Repository { parent, name }
    }

    pub fn service_definition(&self) -> PathBuf {
        let mut path = self.path();
        path.push("locenv-service.yml");
        path
    }

    pub fn path(&self) -> PathBuf {
        let mut path = self.parent.path();
        path.push(self.name.as_ref());
        path
    }
}
