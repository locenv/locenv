use libloading::{Library, Symbol};
use lua::{
    lua_Alloc, lua_CFunction, lua_Integer, lua_Number, lua_Reader, lua_State, lua_Unsigned,
    lua_Writer, size_t, lua_KContext, lua_KFunction,
};
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
    lua_pushvfstring: unsafe { std::mem::transmute(lua::lua_pushvfstring as *const ()) },
    lua_createtable: lua::lua_createtable,
    lua_newuserdatauv: lua::lua_newuserdatauv,
    lua_settable: lua::lua_settable,
    lua_rawset: lua::lua_rawset,
    lua_seti: lua::lua_seti,
    lua_rawseti: lua::lua_rawseti,
    lua_setfield: lua::lua_setfield,
    lua_rawsetp: lua::lua_rawsetp,
    lua_setmetatable: lua::lua_setmetatable,
    lua_setiuservalue: lua::lua_setiuservalue,
    lua_iscfunction: lua::lua_iscfunction,
    lua_isinteger: lua::lua_isinteger,
    lua_isnumber: lua::lua_isnumber,
    lua_isstring: lua::lua_isstring,
    lua_isuserdata: lua::lua_isuserdata,
    lua_type: lua::lua_type,
    lua_typename: lua::lua_typename,
    lua_getmetatable: lua::lua_getmetatable,
    lua_toboolean: lua::lua_toboolean,
    lua_tocfunction: lua::lua_tocfunction,
    lua_tointegerx: lua::lua_tointegerx,
    lua_tolstring: lua::lua_tolstring,
    lua_tonumberx: lua::lua_tonumberx,
    lua_topointer: lua::lua_topointer,
    lua_tothread: lua::lua_tothread,
    lua_touserdata: lua::lua_touserdata,
    lua_geti: lua::lua_geti,
    lua_rawgeti: lua::lua_rawgeti,
    lua_gettable: lua::lua_gettable,
    lua_rawget: lua::lua_rawget,
    lua_getfield: lua::lua_getfield,
    lua_rawgetp: lua::lua_rawgetp,
    lua_next: lua::lua_next,
    lua_getiuservalue: lua::lua_getiuservalue,
    lua_getglobal: lua::lua_getglobal,
    lua_setglobal: lua::lua_setglobal,
    lua_gettop: lua::lua_gettop,
    lua_settop: lua::lua_settop,
    lua_callk: lua::lua_callk,
    lua_pcallk: lua::lua_pcallk,
    lua_error: lua::lua_error,
    lua_warning: lua::lua_warning,
    lua_checkstack: lua::lua_checkstack,
    lua_absindex: lua::lua_absindex,
    lua_copy: lua::lua_copy,
    lua_rotate: lua::lua_rotate,
    lua_len: lua::lua_len,
    lua_rawlen: lua::lua_rawlen,
    lua_compare: lua::lua_compare,
    lua_rawequal: lua::lua_rawequal,
    lua_arith: lua::lua_arith,
    lua_concat: lua::lua_concat,
    lua_load: lua::lua_load,
    lua_dump: lua::lua_dump,
    lua_toclose: lua::lua_toclose,
    lua_closeslot: lua::lua_closeslot,
    lua_stringtonumber: lua::lua_stringtonumber,
    lua_getallocf: lua::lua_getallocf,
    lua_gc: lua::lua_gc,
    lua_version: lua::lua_version,
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
        unsafe extern "C" fn(*mut lua_State, *const c_char, *mut c_void) -> *const c_char,
    lua_createtable: unsafe extern "C" fn(*mut lua_State, c_int, c_int),
    lua_newuserdatauv: unsafe extern "C" fn(*mut lua_State, size_t, c_int) -> *mut c_void,

    lua_settable: unsafe extern "C" fn(*mut lua_State, c_int),
    lua_rawset: unsafe extern "C" fn(*mut lua_State, c_int),
    lua_seti: unsafe extern "C" fn(*mut lua_State, c_int, lua_Integer),
    lua_rawseti: unsafe extern "C" fn(*mut lua_State, c_int, lua_Integer),
    lua_setfield: unsafe extern "C" fn(*mut lua_State, c_int, *const c_char),
    lua_rawsetp: unsafe extern "C" fn(*mut lua_State, c_int, *const c_void),
    lua_setmetatable: unsafe extern "C" fn(*mut lua_State, c_int) -> c_int,
    lua_setiuservalue: unsafe extern "C" fn(*mut lua_State, c_int, c_int) -> c_int,

    lua_iscfunction: unsafe extern "C" fn(*mut lua_State, c_int) -> c_int,
    lua_isinteger: unsafe extern "C" fn(*mut lua_State, c_int) -> c_int,
    lua_isnumber: unsafe extern "C" fn(*mut lua_State, c_int) -> c_int,
    lua_isstring: unsafe extern "C" fn(*mut lua_State, c_int) -> c_int,
    lua_isuserdata: unsafe extern "C" fn(*mut lua_State, c_int) -> c_int,
    lua_type: unsafe extern "C" fn(*mut lua_State, c_int) -> c_int,
    lua_typename: unsafe extern "C" fn(*mut lua_State, c_int) -> *const c_char,
    lua_getmetatable: unsafe extern "C" fn(*mut lua_State, c_int) -> c_int,

    lua_toboolean: unsafe extern "C" fn(*mut lua_State, c_int) -> c_int,
    lua_tocfunction: unsafe extern "C" fn(*mut lua_State, c_int) -> lua_CFunction,
    lua_tointegerx: unsafe extern "C" fn(*mut lua_State, c_int, *mut c_int) -> lua_Integer,
    lua_tolstring: unsafe extern "C" fn(*mut lua_State, c_int, *mut size_t) -> *const c_char,
    lua_tonumberx: unsafe extern "C" fn(*mut lua_State, c_int, *mut c_int) -> lua_Number,
    lua_topointer: unsafe extern "C" fn(*mut lua_State, c_int) -> *const c_void,
    lua_tothread: unsafe extern "C" fn(*mut lua_State, c_int) -> *mut lua_State,
    lua_touserdata: unsafe extern "C" fn(*mut lua_State, c_int) -> *mut c_void,

    lua_geti: unsafe extern "C" fn(*mut lua_State, c_int, lua_Integer) -> c_int,
    lua_rawgeti: unsafe extern "C" fn(*mut lua_State, c_int, lua_Integer) -> c_int,
    lua_gettable: unsafe extern "C" fn(*mut lua_State, c_int) -> c_int,
    lua_rawget: unsafe extern "C" fn(*mut lua_State, c_int) -> c_int,
    lua_getfield: unsafe extern "C" fn(*mut lua_State, c_int, *const c_char) -> c_int,
    lua_rawgetp: unsafe extern "C" fn(*mut lua_State, c_int, *const c_void) -> c_int,
    lua_next: unsafe extern "C" fn(*mut lua_State, c_int) -> c_int,
    lua_getiuservalue: unsafe extern "C" fn(*mut lua_State, c_int, c_int) -> c_int,

    lua_getglobal: unsafe extern "C" fn(*mut lua_State, *const c_char) -> c_int,
    lua_setglobal: unsafe extern "C" fn(*mut lua_State, *const c_char),

    lua_gettop: unsafe extern "C" fn(*mut lua_State) -> c_int,
    lua_settop: unsafe extern "C" fn(*mut lua_State, c_int),

    lua_callk: unsafe extern "C" fn(*mut lua_State, c_int, c_int, lua_KContext, lua_KFunction),
    lua_pcallk: unsafe extern "C" fn(*mut lua_State, c_int, c_int, c_int, lua_KContext, lua_KFunction) -> c_int,
    lua_error: unsafe extern "C" fn(*mut lua_State) -> c_int,
    lua_warning: unsafe extern "C" fn(*mut lua_State, *const c_char, c_int),

    lua_checkstack: unsafe extern "C" fn(*mut lua_State, c_int) -> c_int,
    lua_absindex: unsafe extern "C" fn(*mut lua_State, c_int) -> c_int,
    lua_copy: unsafe extern "C" fn(*mut lua_State, c_int, c_int),
    lua_rotate: unsafe extern "C" fn(*mut lua_State, c_int, c_int),

    lua_len: unsafe extern "C" fn(*mut lua_State, c_int),
    lua_rawlen: unsafe extern "C" fn(*mut lua_State, c_int) -> lua_Unsigned,
    lua_compare: unsafe extern "C" fn(*mut lua_State, c_int, c_int, c_int) -> c_int,
    lua_rawequal: unsafe extern "C" fn(*mut lua_State, c_int, c_int) -> c_int,

    lua_arith: unsafe extern "C" fn(*mut lua_State, c_int),
    lua_concat: unsafe extern "C" fn(*mut lua_State, c_int),

    lua_load: unsafe extern "C" fn(
        *mut lua_State,
        lua_Reader,
        *mut c_void,
        *const c_char,
        *const c_char,
    ) -> c_int,
    lua_dump: unsafe extern "C" fn(*mut lua_State, lua_Writer, *mut c_void, c_int) -> c_int,

    lua_toclose: unsafe extern "C" fn(*mut lua_State, c_int),
    lua_closeslot: unsafe extern "C" fn(*mut lua_State, c_int),

    lua_stringtonumber: unsafe extern "C" fn(*mut lua_State, *const c_char) -> size_t,
    lua_getallocf: unsafe extern "C" fn(*mut lua_State, *mut *mut c_void) -> lua_Alloc,
    lua_gc: unsafe extern "C" fn(*mut lua_State, c_int, ...) -> c_int,
    lua_version: unsafe extern "C" fn(*mut lua_State) -> lua_Number,
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
