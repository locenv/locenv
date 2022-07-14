use super::Runtime;
use fmap_macros::Directory;
use std::borrow::Cow;
use std::marker::PhantomData;
use std::path::PathBuf;

/// Represents where to store service configurations.
pub struct Configurations<'context> {
    parent: Runtime<'context>,
    name: &'static str,
}

impl<'context> Configurations<'context> {
    pub(super) fn new(parent: Runtime<'context>, name: &'static str) -> Self {
        Self { parent, name }
    }

    pub fn by_name<'name>(self, name: Cow<'name, str>) -> Configuration<'context, 'name> {
        Configuration::new(self, name)
    }

    pub fn path(&self) -> PathBuf {
        let mut path = self.parent.path();
        path.push(self.name);
        path
    }
}

/// Represents a configuration how to build and how to start the service.
#[derive(Directory)]
pub struct Configuration<'context, 'name> {
    parent: Configurations<'context>,
    name: Cow<'name, str>,

    #[directory(name = ".locenv", pub)]
    build_state: PhantomData<BuildState<'context, 'name>>,

    #[placeholder(name = "locenv-service.yml", pub)]
    service_definition: PhantomData<()>,
}

impl<'context, 'name> Configuration<'context, 'name> {
    fn new(parent: Configurations<'context>, name: Cow<'name, str>) -> Self {
        Self {
            parent,
            name,
            build_state: PhantomData,
            service_definition: PhantomData,
        }
    }

    pub fn path(&self) -> PathBuf {
        let mut path = self.parent.path();
        path.push(self.name.as_ref());
        path
    }
}

/// Represents the current build state of the configuration.
#[derive(Directory)]
pub struct BuildState<'context, 'name> {
    parent: Configuration<'context, 'name>,
    name: &'static str,

    #[file(pub, kebab)]
    built_time: PhantomData<fmap::TimestampFile>,
}

impl<'context, 'name> BuildState<'context, 'name> {
    fn new(parent: Configuration<'context, 'name>, name: &'static str) -> Self {
        Self {
            parent,
            name,
            built_time: PhantomData,
        }
    }

    pub fn path(&self) -> PathBuf {
        let mut path = self.parent.path();
        path.push(self.name);
        path
    }
}

/// Represents where to store data of all instances.
pub struct Datas<'context> {
    parent: Runtime<'context>,
    name: &'static str,
}

impl<'context> Datas<'context> {
    pub(super) fn new(parent: Runtime<'context>, name: &'static str) -> Self {
        Self { parent, name }
    }

    pub fn by_name<'name>(self, name: Cow<'name, str>) -> Data<'context, 'name> {
        Data::new(self, name)
    }

    pub fn path(&self) -> PathBuf {
        let mut path = self.parent.path();
        path.push(self.name);
        path
    }
}

/// Represents where to store instance data. This is where the instance will write any data.
pub struct Data<'context, 'name> {
    parent: Datas<'context>,
    name: Cow<'name, str>,
}

impl<'context, 'name> Data<'context, 'name> {
    fn new(parent: Datas<'context>, name: Cow<'name, str>) -> Self {
        Self { parent, name }
    }

    pub fn path(&self) -> PathBuf {
        let mut path = self.parent.path();
        path.push(self.name.as_ref());
        path
    }
}

/// Represents a location to store data for a service manager.
#[derive(Directory)]
pub struct ServiceManager<'context> {
    parent: Runtime<'context>,
    name: &'static str,

    #[file(pub)]
    port: PhantomData<fmap::TextFile<u16>>,
}

impl<'context> ServiceManager<'context> {
    pub(super) fn new(parent: Runtime<'context>, name: &'static str) -> Self {
        Self {
            parent,
            name,
            port: PhantomData,
        }
    }

    pub fn path(&self) -> PathBuf {
        let mut path = self.parent.path();
        path.push(self.name);
        path
    }
}
