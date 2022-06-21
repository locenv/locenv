use context::Context;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::path::PathBuf;

pub mod definition;
pub mod instance;

pub struct Module {
    path: PathBuf,
}

#[derive(Debug)]
pub enum FindError {
    NotFound(PathBuf),
}

// Module

impl Module {
    pub fn find(context: &Context, name: &str) -> Result<Self, FindError> {
        let c = context.modules().by_name(name);
        let p = c.definition();

        if !p.is_file() {
            return Err(FindError::NotFound(p));
        }

        Ok(Module { path: c.path() })
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
            Self::NotFound(d) => write!(f, "Module definition {} does not exists", d.display()),
        }
    }
}
