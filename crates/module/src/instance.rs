use libloading::Library;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

pub struct Instance {
    lib: Library,
}

#[derive(Debug)]
pub enum LoadError {
    LibraryLoadError(PathBuf, libloading::Error),
}

// Instance

impl Instance {
    pub(super) fn load<F: AsRef<Path>>(file: F) -> Result<Self, LoadError> {
        // Append extension.
        let full = file.as_ref().with_extension(if cfg!(linux) {
            "so"
        } else if cfg!(macos) {
            "dylib"
        } else if cfg!(windows) {
            "dll"
        } else {
            panic!("The target platform is not supported")
        });

        // Load.
        let lib = match unsafe { Library::new(&full) } {
            Ok(r) => r,
            Err(e) => return Err(LoadError::LibraryLoadError(full, e)),
        };

        Ok(Instance { lib })
    }
}

// LoadError

impl Error for LoadError {}

impl Display for LoadError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::LibraryLoadError(p, e) => write!(f, "Failed to load {}: {}", p.display(), e),
        }
    }
}
