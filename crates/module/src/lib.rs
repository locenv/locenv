use config::module::ModuleDefinition;
use context::Context;
use instance::{Instance, LoadError};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::path::PathBuf;

pub mod instance;

pub struct Module {
    path: PathBuf,
    name: String,
    file: String,
}

#[derive(Debug)]
pub enum FindError {
    DefinitionLoadError {
        file: PathBuf,
        error: config::FromFileError,
    },
}

// Module

impl Module {
    pub fn find(context: &Context, name: &str) -> Result<Self, FindError> {
        let context = context.modules().by_name(name);

        // Load module definition.
        let path = context.definition();
        let defs = match ModuleDefinition::from_file(&path) {
            Ok(r) => r,
            Err(e) => {
                return Err(FindError::DefinitionLoadError {
                    file: path,
                    error: e,
                })
            }
        };

        Ok(Module {
            path: context.path(),
            name: defs.name,
            file: defs.file,
        })
    }

    pub fn load(&self) -> Result<Instance, LoadError> {
        Instance::load(self.path.join(&self.file))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

// FindError

impl Error for FindError {}

impl Display for FindError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::DefinitionLoadError { file, error } => {
                write!(f, "Failed to load {}: {}", file.display(), error)
            }
        }
    }
}
