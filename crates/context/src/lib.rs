use self::modules::Modules;
use self::project::Project;
use self::runtime::Runtime;
use std::env::VarError;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

pub mod modules;
pub mod project;
pub mod runtime;

pub struct Context {
    project: PathBuf,
    runtime: PathBuf,
    prefix: PathBuf,
}

#[derive(Debug)]
pub enum NewError {
    NoPrefixEnv(String),
    PrefixEnvNotUnicode(String),
}

// Context

impl Context {
    pub fn new<P: Into<PathBuf>>(project: P) -> Result<Self, NewError> {
        let owned = project.into();

        // Get path to global data.
        let var = "LOCENV_PREFIX";
        let prefix = match std::env::var(var) {
            Ok(r) => PathBuf::from(r),
            Err(e) => match e {
                VarError::NotPresent => return Err(NewError::NoPrefixEnv(var.into())),
                VarError::NotUnicode(_) => return Err(NewError::PrefixEnvNotUnicode(var.into())),
            },
        };

        // Construct context.
        let result = Context {
            prefix,
            runtime: owned.join(".locenv"),
            project: owned,
        };

        Ok(result)
    }

    pub fn project(&self) -> Project {
        Project::new(&self.project)
    }

    pub fn runtime(&self) -> Runtime {
        Runtime::new(&self.runtime)
    }

    pub fn modules(&self) -> Modules {
        Modules::new(&self.prefix, "module")
    }
}

// NewError

impl Error for NewError {}

impl Display for NewError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::NoPrefixEnv(v) => write!(f, "No environment variable {} has been set", &v),
            Self::PrefixEnvNotUnicode(v) => write!(
                f,
                "Environment variable {} contains invalid unicode data",
                v
            ),
        }
    }
}
