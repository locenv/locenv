use context::Context;
use lua::LUA_OK;
use module::Module;
use std::error::Error;
use std::ffi::CString;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

pub struct Engine<'context> {
    lua: *mut lua::lua_State,
    phantom: PhantomData<&'context mut lua::lua_State>,
}

#[derive(Debug)]
pub enum RunError {
    LoadError(String),
    ExecError(String),
    MissingModule(String),
}

// Engine

impl<'context> Engine<'context> {
    pub fn new(context: &'context Context) -> Self {
        // Allocate state.
        let l = unsafe { lua::luaL_newstate() };

        if l.is_null() {
            panic!("Failed to create Lua engine due to insufficient memory");
        }

        // Setup base library.
        unsafe { lua::luaopen_base(l) };

        // Setup package library.
        unsafe { lua::luaopen_package(l) };
        lua::get_field(l, -1, "searchers");

        for i in (2..=4).rev() {
            // Remove all package.searchers except the first one.
            unsafe { lua::lua_pushnil(l) };
            unsafe { lua::lua_seti(l, -2, i) };
        }

        unsafe { lua::lua_pushlightuserdata(l, context as *const _ as *mut lua::c_void) };
        unsafe { lua::lua_pushcclosure(l, Self::module_searcher, 1) };
        unsafe { lua::lua_seti(l, -2, 2) };

        lua::pop(l, 1);

        Engine {
            lua: l,
            phantom: PhantomData,
        }
    }

    pub fn run(&mut self, script: &str) -> Result<(), RunError> {
        // Load script.
        let lua = CString::new(script).unwrap();
        let status = unsafe { lua::luaL_loadstring(self.lua, lua.as_ptr()) };

        if status != LUA_OK {
            return Err(RunError::LoadError(lua::pop_string(self.lua).unwrap()));
        }

        // Run script.
        let status = unsafe { lua::lua_pcallk(self.lua, 0, 0, 0, 0, None) };

        if status != LUA_OK {
            return Err(RunError::ExecError(lua::pop_string(self.lua).unwrap()));
        }

        Ok(())
    }

    unsafe extern "C" fn module_searcher(l: *mut lua::lua_State) -> lua::c_int {
        let context = &*(lua::lua_topointer(l, lua::LUA_REGISTRYINDEX - 1) as *const Context);
        let name = lua::check_string(l, 1).unwrap();

        // Find the module.
        let module = match Module::find(context, &name) {
            Ok(r) => r,
            Err(e) => match e {
                module::FindError::DefinitionLoadError { file, error } => match error {
                    config::FromFileError::OpenFailed(e) => {
                        lua::push_string(l, &format!("cannot open {}: {}", file.display(), e));
                        return 1;
                    }
                    config::FromFileError::ParseFailed(e) => {
                        lua::push_string(l, &format!("cannot parse {}: {}", file.display(), e));
                        return 1;
                    }
                },
            },
        };

        // Load the module.
        let instance = match module.load() {
            Ok(r) => r,
            Err(e) => match e {
                module::instance::LoadError::LibraryLoadError(f, e) => {
                    lua::push_string(l, &format!("cannot load {}: {}", f.display(), e));
                    return 1;
                }
            },
        };

        lua::lua_pushnil(l);
        1
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
            Self::MissingModule(name) => {
                write!(f, "This system does not have module '{}' installed", name)
            }
        }
    }
}
