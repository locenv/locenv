use libloading::{Library, Symbol};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

const API_TABLE: ApiTable = ApiTable {};

pub struct Instance {
    library: Library,
}

#[derive(Debug)]
pub enum LoadError {
    LibraryLoadError(PathBuf, libloading::Error),
}

#[derive(Debug)]
pub enum BootstrapError {
    GetFunctionFailed(String, Box<dyn Error>),
}

#[repr(C)]
struct ApiTable {}

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
        let library = match unsafe { Library::new(&full) } {
            Ok(r) => r,
            Err(e) => return Err(LoadError::LibraryLoadError(full, e)),
        };

        Ok(Instance { library })
    }

    pub fn bootstrap(&self, lua: *mut lua::lua_State) -> Result<lua::c_int, BootstrapError> {
        let bootstrap: Symbol<
            unsafe extern "C" fn(*mut lua::lua_State, *const ApiTable) -> lua::c_int,
        > = match unsafe { self.library.get(b"bootstrap\0") } {
            Ok(r) => r,
            Err(e) => {
                return Err(BootstrapError::GetFunctionFailed(
                    "bootstrap".into(),
                    e.into(),
                ))
            }
        };

        Ok(unsafe { bootstrap(lua, &API_TABLE) })
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

// BootstrapError

impl Error for BootstrapError {}

impl Display for BootstrapError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            BootstrapError::GetFunctionFailed(n, e) => {
                write!(f, "Failed to get the address of '{}' function: {}", n, e)
            }
        }
    }
}
