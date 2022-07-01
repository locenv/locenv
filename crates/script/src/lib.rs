use context::Context;
use module::Module;
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::CString;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

pub struct Engine<'context> {
    lua: *mut lua::lua_State,
    loaded_modules: HashMap<Module<'context, 'static>, module::instance::Instance>,
    phantom: PhantomData<&'context mut lua::lua_State>,
}

#[derive(Debug)]
pub enum RunError {
    LoadError(String),
    ExecError(String),
}

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
        unsafe { lua::luaopen_base(lua) };

        // Setup package library.
        unsafe { lua::luaopen_package(lua) };
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

        // Load and bootstrap the module.
        let instance = match module.load() {
            Ok(r) => r,
            Err(e) => match e {
                module::instance::LoadError::LibraryLoadError(f, e) => {
                    lua::push_string(lua, &format!("cannot load {}: {}", f.display(), e));
                    return 1;
                }
            },
        };

        let returns = match instance.bootstrap(lua) {
            Ok(r) => r,
            Err(e) => match e {
                module::instance::BootstrapError::GetFunctionFailed(f, e) => {
                    lua::push_string(lua, &format!("cannot find exported function {}: {}", f, e));
                    return 1;
                }
            },
        };

        // Keep loaded module until the engine is dropped even if the bootstrap function was failed to make sure Lue does not have a dangling pointer in some
        // cases.
        // TODO: Use try_insert once https://github.com/rust-lang/rust/issues/82766 is stable.
        if self.loaded_modules.insert(module, instance).is_some() {
            panic!("Some module was loaded twice somehow")
        }

        match returns {
            v @ (1 | 2) => v,
            _ => panic!(
                "An unexpected value was returned from bootstrapping function of the loaed module"
            ),
        }
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
