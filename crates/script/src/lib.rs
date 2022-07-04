use context::Context;
use lua::{
    lua_Alloc, lua_CFunction, lua_Integer, lua_KContext, lua_KFunction, lua_Number, lua_Reader,
    lua_State, lua_Unsigned, lua_Writer, size_t,
};
use module::Module;
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::{c_void, CString};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::os::raw::{c_char, c_int};

pub struct Engine<'context> {
    lua: *mut lua::lua_State,
    loaded_modules: HashMap<Module<'context, 'static>, Option<libloading::Library>>,
    phantom: PhantomData<&'context mut lua::lua_State>,
}

#[derive(Debug)]
pub enum RunError {
    LoadError(String),
    ExecError(String),
}

#[repr(C)]
struct ApiTable {
    revision: u32,

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
    lua_pcallk: unsafe extern "C" fn(
        *mut lua_State,
        c_int,
        c_int,
        c_int,
        lua_KContext,
        lua_KFunction,
    ) -> c_int,
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

const API_TABLE: ApiTable = ApiTable {
    revision: 0,
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

// Engine

impl<'context> Engine<'context> {
    pub fn new(context: &'context Context) -> Self {
        // Allocate state.
        let lua = unsafe { lua::luaL_newstate() };

        if lua.is_null() {
            panic!("Failed to create Lua engine due to insufficient memory");
        }

        let mut engine = Engine {
            lua,
            loaded_modules: HashMap::new(),
            phantom: PhantomData,
        };

        // Setup base library.
        unsafe {
            lua::luaL_requiref(
                lua,
                lua::LUA_GNAME.as_ptr() as *const _,
                Some(lua::luaopen_base),
                1,
            )
        };
        lua::pop(lua, 1);

        // Setup package library.
        unsafe {
            lua::luaL_requiref(
                lua,
                lua::LUA_LOADLIBNAME.as_ptr() as *const _,
                Some(lua::luaopen_package),
                1,
            )
        };
        lua::get_field(lua, -1, "searchers");

        for i in (2..=4).rev() {
            // Remove all package.searchers except the first one.
            unsafe { lua::lua_pushnil(lua) };
            unsafe { lua::lua_seti(lua, -2, i) };
        }

        lua::push_closure(lua, |lua| engine.module_searcher(lua, context));
        unsafe { lua::lua_seti(lua, -2, 2) };
        lua::pop(lua, 1); // Pop searchers.

        engine
    }

    pub fn run(&mut self, script: &str) -> Result<(), RunError> {
        // Load script.
        let lua = CString::new(script).unwrap();
        let status = unsafe { lua::luaL_loadstring(self.lua, lua.as_ptr()) };

        if status != 0 {
            return Err(RunError::LoadError(lua::pop_string(self.lua).unwrap()));
        }

        // Run script.
        let status = unsafe { lua::lua_pcallk(self.lua, 0, 0, 0, 0, None) };

        if status != 0 {
            return Err(RunError::ExecError(lua::pop_string(self.lua).unwrap()));
        }

        Ok(())
    }

    fn module_searcher(
        &mut self,
        lua: *mut lua::lua_State,
        context: &'context Context,
    ) -> lua::c_int {
        let name = lua::check_string(lua, 1).unwrap();

        // Find the module.
        let module = match Module::find(context, Cow::Owned(name)) {
            Ok(r) => r,
            Err(e) => match e {
                module::FindError::NotInstalled(p) => {
                    lua::push_string(
                        lua,
                        &format!("the module is not installed in {}", p.display()),
                    );
                    return 1;
                }
                module::FindError::LoadDefinitionFailed(f, e) => match e {
                    config::FromFileError::OpenFailed(e) => {
                        lua::push_string(lua, &format!("cannot open {}: {}", f.display(), e));
                        return 1;
                    }
                    config::FromFileError::ParseFailed(e) => {
                        lua::push_string(lua, &format!("cannot parse {}: {}", f.display(), e));
                        return 1;
                    }
                },
            },
        };

        // Load the module.
        let library: Option<libloading::Library> = match &module.definition().program {
            config::module::Program::Script(file) => {
                let path = module.path().join(file);
                let file = CString::new(path.to_str().unwrap()).unwrap();
                let status = unsafe { lua::luaL_loadfilex(lua, file.as_ptr(), std::ptr::null()) };

                if status != 0 {
                    // luaL_loadfilex already pushed the error message.
                    return 1;
                }

                unsafe { lua::lua_pushstring(lua, file.as_ptr()) };

                None
            }
            config::module::Program::Binary(program) => match program.current() {
                Some(file) => {
                    // Load the module.
                    let path = module.path().join(file);
                    let program = match unsafe { libloading::Library::new(&path) } {
                        Ok(r) => r,
                        Err(e) => {
                            lua::push_string(
                                lua,
                                &format!("cannot load {}: {}", path.display(), e),
                            );
                            return 1;
                        }
                    };

                    // Get bootstrap function.
                    let bootstrap = match unsafe {
                        program.get::<unsafe extern "C" fn(*mut lua_State) -> c_int>(b"bootstrap\0")
                    } {
                        Ok(r) => unsafe { r.into_raw().into_raw() },
                        Err(e) => {
                            lua::push_string(
                                lua,
                                &format!(
                                    "cannot find bootstrap function in {}: {}",
                                    path.display(),
                                    e
                                ),
                            );
                            return 1;
                        }
                    };

                    // Push loader.
                    unsafe {
                        lua::lua_pushcclosure(lua, Some(std::mem::transmute(bootstrap)), 0);
                        lua::lua_pushlightuserdata(
                            lua,
                            std::mem::transmute(&API_TABLE as *const _),
                        );
                    }

                    Some(program)
                }
                None => {
                    lua::push_string(lua, "the module cannot run on this platform");
                    return 1;
                }
            },
        };

        // Add module to loaded table.
        // TODO: Use try_insert once https://github.com/rust-lang/rust/issues/82766 is stable.
        if self.loaded_modules.insert(module, library).is_some() {
            panic!("Some module was loaded twice somehow")
        }

        2
    }
}

impl<'context> Drop for Engine<'context> {
    fn drop(&mut self) {
        unsafe { lua::lua_close(self.lua) };
    }
}

// RunError

impl Error for RunError {}

impl Display for RunError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::LoadError(e) => write!(f, "Failed to load script: {}", e),
            Self::ExecError(e) => write!(f, "Failed to execute script: {}", e),
        }
    }
}
