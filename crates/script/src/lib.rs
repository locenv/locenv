use context::Context;
use lua::lua_State;
use module::Module;
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::CString;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::os::raw::c_int;

mod api;

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
                            std::mem::transmute(&api::TABLE as *const _),
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
