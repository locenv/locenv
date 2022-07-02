use libloading::{Library, Symbol};
use lua::{__va_list_tag, lua_CFunction, lua_Integer, lua_Number, lua_State, size_t};
use std::error::Error;
use std::ffi::c_void;
use std::fmt::{Display, Formatter};
use std::os::raw::{c_char, c_int};
use std::path::{Path, PathBuf};

const API_TABLE: ApiTable = ApiTable {
    lua_pushboolean: lua::lua_pushboolean,
    lua_pushcclosure: lua::lua_pushcclosure,
    lua_pushfstring: lua::lua_pushfstring,
    lua_pushinteger: lua::lua_pushinteger,
    lua_pushlightuserdata: lua::lua_pushlightuserdata,
    lua_pushlstring: lua::lua_pushlstring,
    lua_pushnil: lua::lua_pushnil,
    lua_pushnumber: lua::lua_pushnumber,
    lua_pushstring: lua::lua_pushstring,
    lua_pushthread: lua::lua_pushthread,
    lua_pushvalue: lua::lua_pushvalue,
    lua_pushvfstring: lua::lua_pushvfstring,
};

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
struct ApiTable {
    lua_pushboolean: unsafe extern "C" fn(*mut lua_State, c_int),
    lua_pushcclosure: unsafe extern "C" fn(*mut lua_State, lua_CFunction, c_int),
    lua_pushfstring: unsafe extern "C" fn(*mut lua_State, *const c_char, ...) -> *const c_char,
    lua_pushinteger: unsafe extern "C" fn(*mut lua_State, lua_Integer),
    lua_pushlightuserdata: unsafe extern "C" fn(*mut lua_State, *mut c_void),
    lua_pushlstring: unsafe extern "C" fn(*mut lua_State, *const c_char, size_t) -> *const c_char,
    lua_pushnil: unsafe extern "C" fn(*mut lua_State),
    lua_pushnumber: unsafe extern "C" fn(*mut lua_State, lua_Number),
    lua_pushstring: unsafe extern "C" fn(*mut lua_State, *const c_char) -> *const c_char,
    lua_pushthread: unsafe extern "C" fn(*mut lua_State) -> c_int,
    lua_pushvalue: unsafe extern "C" fn(*mut lua_State, c_int),
    lua_pushvfstring:
        unsafe extern "C" fn(*mut lua_State, *const c_char, *mut __va_list_tag) -> *const c_char,
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
