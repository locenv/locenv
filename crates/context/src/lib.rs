use fmap_macros::Directory;
use std::env::VarError;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

pub mod data;
pub mod runtime;

/// Represents a context to run locenv.
pub struct Context {
    project: PathBuf,
    runtime: PathBuf,
    data: PathBuf,
}

impl Context {
    pub fn new<P: Into<PathBuf>>(project: P) -> Result<Self, ContextError> {
        let project = project.into();

        // Get path to locenv data.
        let var = "LOCENV_DATA";
        let data = match std::env::var(var) {
            Ok(r) => PathBuf::from(r),
            Err(e) => match e {
                VarError::NotPresent => return Err(ContextError::NoPrefixEnv(var.into())),
                VarError::NotUnicode(_) => {
                    return Err(ContextError::PrefixEnvNotUnicode(var.into()))
                }
            },
        };

        // Construct context.
        Ok(Self {
            runtime: project.join(".locenv"),
            project,
            data,
        })
    }

    /// Gets the current project.
    pub fn project(&self) -> Project {
        Project::new(&self.project)
    }

    /// Gets the runtime of the current project.
    pub fn runtime(&self) -> Runtime {
        Runtime::new(&self.runtime)
    }

    /// Get the global data for locenv.
    pub fn data(&self) -> Datas {
        Datas::new(&self.data)
    }
}

/// Represents a project to work on.
#[derive(Directory)]
pub struct Project<'context> {
    path: &'context Path,

    #[placeholder(name = "locenv-services.yml", pub)]
    services: PhantomData<()>,
}

impl<'context> Project<'context> {
    fn new(path: &'context Path) -> Self {
        Project {
            path,
            services: PhantomData,
        }
    }

    pub fn path(&self) -> PathBuf {
        self.path.into()
    }
}

/// Represents a runtime directory of the project.
#[derive(Directory)]
pub struct Runtime<'context> {
    path: &'context Path,

    #[directory(pub)]
    configurations: PhantomData<self::runtime::Configurations<'context>>,

    #[directory(pub)]
    data: PhantomData<self::runtime::Datas<'context>>,

    #[directory(pub, kebab)]
    service_manager: PhantomData<self::runtime::ServiceManager<'context>>,
}

impl<'context> Runtime<'context> {
    fn new(path: &'context Path) -> Self {
        Runtime {
            path,
            configurations: PhantomData,
            data: PhantomData,
            service_manager: PhantomData,
        }
    }

    pub fn path(&self) -> PathBuf {
        self.path.into()
    }
}

/// Represents locenv global data.
#[derive(Directory)]
pub struct Datas<'context> {
    path: &'context Path,

    #[directory(pub)]
    module: PhantomData<self::data::Modules<'context>>,
}

impl<'context> Datas<'context> {
    pub fn new(path: &'context Path) -> Self {
        Self {
            path,
            module: PhantomData,
        }
    }

    pub fn path(&self) -> PathBuf {
        self.path.into()
    }
}

/// Represents the error when instantiate the context.
#[derive(Debug)]
pub enum ContextError {
    NoPrefixEnv(String),
    PrefixEnvNotUnicode(String),
}

impl Error for ContextError {}

impl Display for ContextError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::NoPrefixEnv(v) => write!(f, "No environment variable {} has been set", v),
            Self::PrefixEnvNotUnicode(v) => write!(
                f,
                "Environment variable {} contains invalid unicode data",
                v
            ),
        }
    }
}
