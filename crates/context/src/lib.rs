use dirtree_macros::Directory;
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
        Ok(Self { project, data })
    }

    /// Gets the current project.
    pub fn project(&self) -> Project {
        Project::new(&self.project)
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

    #[directory(pub, name = ".locenv")]
    runtime: PhantomData<self::runtime::Runtime<'context>>,

    #[placeholder(pub, name = "locenv-services.yml")]
    services: PhantomData<()>,
}

impl<'context> Project<'context> {
    fn new(path: &'context Path) -> Self {
        Project {
            path,
            runtime: PhantomData,
            services: PhantomData,
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

    #[directory(pub)]
    config: PhantomData<self::data::Configurations<'context>>,
}

impl<'context> Datas<'context> {
    fn new(path: &'context Path) -> Self {
        Self {
            path,
            module: PhantomData,
            config: PhantomData,
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
