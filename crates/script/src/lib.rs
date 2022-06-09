use lua::LUA_OK;
use std::error::Error;
use std::ffi::CString;
use std::fmt::{Display, Formatter};

pub struct Engine {
    lua: *mut lua::lua_State,
}

#[derive(Debug)]
pub enum RunError {
    LoadError(String),
    ExecError(String),
    MissingModule(String),
}

// Engine

impl Engine {
    pub fn new() -> Self {
        // Allocate state.
        let lua = unsafe { lua::luaL_newstate() };

        if lua.is_null() {
            panic!("Failed to create Lua engine due to insufficient memory");
        }

        // Enable core libraries.
        unsafe {
            lua::luaopen_base(lua);
        }

        Engine { lua }
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
}

impl Drop for Engine {
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
