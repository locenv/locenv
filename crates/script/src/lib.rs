use context::Context;
use lua::{
    get_field, luaL_newstate, luaL_requiref, lua_State, lua_pushnil, lua_seti, luaopen_base,
    luaopen_io, luaopen_math, luaopen_os, luaopen_package, luaopen_string, luaopen_table,
    luaopen_utf8, pop, push_closure, LUA_GNAME, LUA_IOLIBNAME, LUA_LOADLIBNAME, LUA_MATHLIBNAME,
    LUA_OSLIBNAME, LUA_STRLIBNAME, LUA_TABLIBNAME, LUA_UTF8LIBNAME,
};
use module::Module;
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::CString;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::mem::transmute;
use std::os::raw::{c_char, c_int};

mod api;

pub struct Engine<'context> {
    lua: *mut lua::lua_State,
    loaded_modules: HashMap<Module<'context, 'static>, Option<NativeData>>,
    phantom: PhantomData<&'context mut lua::lua_State>,
}

#[derive(Debug)]
pub enum RunError {
    LoadError(String),
    ExecError(String),
}

#[allow(dead_code)]
struct NativeData {
    library: libloading::Library,
    name: Box<CString>,
    data: Box<LoaderData>,
}

#[repr(C)]
struct LoaderData {
    name: *const c_char,
    api: *const api::Table,
}

// Engine

impl<'context> Engine<'context> {
    pub fn new(context: &'context Context) -> Self {
        // Allocate state.
        let lua = unsafe { luaL_newstate() };

        if lua.is_null() {
            panic!("Failed to create Lua engine due to insufficient memory");
        }

        let mut engine = Engine {
            lua,
            loaded_modules: HashMap::new(),
            phantom: PhantomData,
        };

        let install = |name: &[u8], loader: unsafe extern "C" fn(*mut lua_State) -> c_int| unsafe {
            luaL_requiref(lua, transmute(name.as_ptr()), Some(loader), 1)
        };

        // Setup base library.
        install(LUA_GNAME, luaopen_base);
        pop(lua, 1);

        // Setup package library.
        install(LUA_LOADLIBNAME, luaopen_package);
        get_field(lua, -1, "searchers");

        for i in (2..=4).rev() {
            // Remove all package.searchers except the first one.
            unsafe { lua_pushnil(lua) };
            unsafe { lua_seti(lua, -2, i) };
        }

        push_closure(lua, |lua| engine.module_searcher(lua, context));
        unsafe { lua_seti(lua, -2, 2) };
        pop(lua, 2); // Pop searchers and module.

        // Setup table library.
        install(LUA_TABLIBNAME, luaopen_table);
        pop(lua, 1);

        // Setup I/O library.
        install(LUA_IOLIBNAME, luaopen_io);
        pop(lua, 1);

        // Setup OS library.
        install(LUA_OSLIBNAME, luaopen_os);
        pop(lua, 1);

        // Setup string library.
        install(LUA_STRLIBNAME, luaopen_string);
        pop(lua, 1);

        // Setup math library.
        install(LUA_MATHLIBNAME, luaopen_math);
        pop(lua, 1);

        // Setup UTF-8 library.
        install(LUA_UTF8LIBNAME, luaopen_utf8);
        pop(lua, 1);

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
        let native: Option<NativeData> = match &module.definition().program {
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
                    let library = match unsafe { libloading::Library::new(&path) } {
                        Ok(r) => r,
                        Err(e) => {
                            lua::push_string(
                                lua,
                                &format!("cannot load {}: {}", path.display(), e),
                            );
                            return 1;
                        }
                    };

                    // Get loader function.
                    let loader = match unsafe {
                        library.get::<unsafe extern "C" fn(*mut lua_State) -> c_int>(b"loader\0")
                    } {
                        Ok(r) => unsafe { r.into_raw().into_raw() },
                        Err(e) => {
                            lua::push_string(
                                lua,
                                &format!(
                                    "cannot find loader function in {}: {}",
                                    path.display(),
                                    e
                                ),
                            );
                            return 1;
                        }
                    };

                    // Allocate loader data.
                    let name = Box::new(CString::new(module.definition().name.as_str()).unwrap());
                    let data = Box::new(LoaderData {
                        name: name.as_ptr(),
                        api: &api::TABLE,
                    });

                    // Push loader.
                    unsafe {
                        lua::lua_pushcclosure(lua, Some(transmute(loader)), 0);
                        lua::lua_pushlightuserdata(lua, transmute(&*data));
                    }

                    Some(NativeData {
                        library,
                        name,
                        data,
                    })
                }
                None => {
                    lua::push_string(lua, "the module cannot run on this platform");
                    return 1;
                }
            },
        };

        // Add module to loaded table.
        // TODO: Use try_insert once https://github.com/rust-lang/rust/issues/82766 is stable.
        if self.loaded_modules.insert(module, native).is_some() {
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
