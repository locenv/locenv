use crate::Project;
use dirtree::{TextFile, TimestampFile};
use dirtree_macros::Directory;
use std::borrow::Cow;
use std::marker::PhantomData;
use std::path::PathBuf;

/// Represents a runtime directory of the project.
#[derive(Directory)]
pub struct Runtime<'context> {
    parent: Project<'context>,
    name: &'static str,

    #[directory(pub)]
    configurations: PhantomData<Configurations<'context>>,

    #[directory(pub)]
    data: PhantomData<Datas<'context>>,

    #[directory(pub, kebab)]
    service_manager: PhantomData<ServiceManager<'context>>,
}

impl<'context> Runtime<'context> {
    pub(super) fn new(parent: Project<'context>, name: &'static str) -> Self {
        Self {
            parent,
            name,
            configurations: PhantomData,
            data: PhantomData,
            service_manager: PhantomData,
        }
    }

    pub fn path(&self) -> PathBuf {
        let mut path = self.parent.path();
        path.push(self.name);
        path
    }
}

/// Represents where to store service configurations.
pub struct Configurations<'context> {
    parent: Runtime<'context>,
    name: &'static str,
}

impl<'context> Configurations<'context> {
    fn new(parent: Runtime<'context>, name: &'static str) -> Self {
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
    built_time: PhantomData<TimestampFile>,
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
    fn new(parent: Runtime<'context>, name: &'static str) -> Self {
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
    pid: PhantomData<TextFile<u32>>,

    #[file(pub)]
    port: PhantomData<TextFile<u16>>,

    #[placeholder(pub, ext = "txt")]
    log: PhantomData<()>,
}

impl<'context> ServiceManager<'context> {
    fn new(parent: Runtime<'context>, name: &'static str) -> Self {
        Self {
            parent,
            name,
            pid: PhantomData,
            port: PhantomData,
            log: PhantomData,
        }
    }

    pub fn path(&self) -> PathBuf {
        let mut path = self.parent.path();
        path.push(self.name);
        path
    }
}
