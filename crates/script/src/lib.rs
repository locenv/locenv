use context::Context;
use libloading::Library;
use lua::{
    luaL_checklstring, luaL_loadfilex, luaL_loadstring, luaL_newstate, luaL_requiref, lua_Integer,
    lua_State, lua_close, lua_getfield, lua_pcallk, lua_pushcclosure, lua_pushlightuserdata,
    lua_pushnil, lua_pushstring, lua_seti, lua_settop, lua_tolstring, lua_touserdata, luaopen_base,
    luaopen_io, luaopen_math, luaopen_os, luaopen_package, luaopen_string, luaopen_table,
    luaopen_utf8, LUA_GNAME, LUA_IOLIBNAME, LUA_LOADLIBNAME, LUA_MATHLIBNAME, LUA_OSLIBNAME,
    LUA_REGISTRYINDEX, LUA_STRLIBNAME, LUA_TABLIBNAME, LUA_UTF8LIBNAME,
};
use module::Module;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::{c_void, CStr, CString};
use std::mem::transmute;
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::ptr::{null, null_mut};

mod api;

macro_rules! cfmt {
    ($($x:tt)*) => {
        CString::new(format!($($x)*)).unwrap()
    }
}

/// Represents an isolated environment to execute Lua script.
pub struct Engine<'context> {
    lua: *mut lua_State, // This one need to drop first so all of __gc method in the module still work correctly.
    loaded_modules: Box<ModuleTable<'context>>,
    working_directory: Vec<u8>,
}

impl<'context> Engine<'context> {
    pub fn new<W: AsRef<Path>>(context: &'context Context, working_directory: W) -> Self {
        // Allocate engine.
        let working_directory = CString::new(working_directory.as_ref().to_str().unwrap()).unwrap();
        let mut engine = Engine {
            lua: null_mut(),
            loaded_modules: Box::new(ModuleTable::new()),
            working_directory: Vec::from(working_directory.as_bytes_with_nul()),
        };

        // Allocate Lua.
        engine.lua = unsafe { luaL_newstate() };

        if engine.lua.is_null() {
            panic!("Failed to create Lua engine due to insufficient memory");
        }

        // Setup base library.
        engine.open_library(LUA_GNAME, luaopen_base);
        engine.pop_stack(1);

        // Setup package library.
        engine.open_library(LUA_LOADLIBNAME, luaopen_package);
        engine.get_field(-1, "searchers");

        for i in (2..=4).rev() {
            // Remove all package.searchers except the first one.
            engine.push_nil();
            engine.set_index(-2, i);
        }

        engine.push_light_userdata(unsafe { transmute(context) });
        engine.push_light_userdata(unsafe { transmute(&*engine.loaded_modules) });
        engine.push_light_userdata(unsafe { transmute(engine.working_directory.as_ptr()) });
        engine.push_fn(Self::module_searcher, 3);

        engine.set_index(-2, 2);
        engine.pop_stack(2); // Pop searchers and module.

        // Setup table library.
        engine.open_library(LUA_TABLIBNAME, luaopen_table);
        engine.pop_stack(1);

        // Setup I/O library.
        engine.open_library(LUA_IOLIBNAME, luaopen_io);
        engine.pop_stack(1);

        // Setup OS library.
        engine.open_library(LUA_OSLIBNAME, luaopen_os);
        engine.pop_stack(1);

        // Setup string library.
        engine.open_library(LUA_STRLIBNAME, luaopen_string);
        engine.pop_stack(1);

        // Setup math library.
        engine.open_library(LUA_MATHLIBNAME, luaopen_math);
        engine.pop_stack(1);

        // Setup UTF-8 library.
        engine.open_library(LUA_UTF8LIBNAME, luaopen_utf8);
        engine.pop_stack(1);

        engine
    }

    pub fn run<A: ToLua>(
        &mut self,
        script: &str,
        argument: Option<&A>,
    ) -> Result<(), RunError<A::Err>> {
        // Load script.
        let script = CString::new(script).unwrap();
        let status = unsafe { luaL_loadstring(self.lua, script.as_ptr()) };

        if status != 0 {
            return Err(RunError::LoadError(self.pop_string().unwrap()));
        }

        // Push arguments.
        let args: c_int = if let Some(a) = argument {
            match a.to_lua(self.lua) {
                Ok(r) => r,
                Err(e) => return Err(RunError::ArgumentError(e)),
            }
        } else {
            0
        };

        // Run script.
        let status = unsafe { lua_pcallk(self.lua, args, 0, 0, 0, None) };

        if status != 0 {
            return Err(RunError::ExecError(self.pop_string().unwrap()));
        }

        Ok(())
    }

    unsafe extern "C" fn module_searcher(lua: *mut lua::lua_State) -> c_int {
        let context: &Context = transmute(lua_touserdata(lua, LUA_REGISTRYINDEX - 1));
        let loaded: &mut ModuleTable = transmute(lua_touserdata(lua, LUA_REGISTRYINDEX - 2));
        let wd: *const c_char = transmute(lua_touserdata(lua, LUA_REGISTRYINDEX - 3));
        let name = luaL_checklstring(lua, 1, null_mut());
        let name = CStr::from_ptr(name).to_str().unwrap();

        // Find the module.
        let module = match Module::find(context, Cow::Owned(name.into())) {
            Ok(r) => r,
            Err(e) => match e {
                module::FindError::NotInstalled(p) => {
                    let message = cfmt!("the module is not installed in {}", p.display());
                    lua_pushstring(lua, message.as_ptr());
                    return 1;
                }
                module::FindError::LoadDefinitionFailed(f, e) => match e {
                    yaml::FileError::OpenFailed(e) => {
                        let message = cfmt!("cannot open {}: {}", f.display(), e);
                        lua_pushstring(lua, message.as_ptr());
                        return 1;
                    }
                    yaml::FileError::ParseFailed(e) => {
                        let message = cfmt!("cannot parse {}: {}", f.display(), e);
                        lua_pushstring(lua, message.as_ptr());
                        return 1;
                    }
                },
            },
        };

        // Load the module.
        let native: Option<Library> = match &module.definition().program {
            module::definition::Program::Script(file) => {
                let path = module.path().join(file);
                let file = CString::new(path.to_str().unwrap()).unwrap();
                let status = luaL_loadfilex(lua, file.as_ptr(), null());

                if status != 0 {
                    // luaL_loadfilex already pushed the error message.
                    return 1;
                }

                lua_pushstring(lua, file.as_ptr());
                None
            }
            module::definition::Program::Binary(program) => match program.current() {
                Some(files) => match files.current() {
                    Some(file) => {
                        let name = CString::new(module.definition().name.as_str()).unwrap();
                        let file = module.path().join(file);
                        let context = api::BootstrapContext {
                            revision: 0,
                            name: name.as_ptr(),
                            locenv: transmute(context),
                            lua,
                            working_directory: wd,
                        };

                        let result = Self::bootstrap_native_module(&context, &file);

                        if result.is_none() {
                            return 1;
                        }

                        result
                    }
                    None => {
                        let message = CString::new("the module cannot run with your CPU").unwrap();
                        lua_pushstring(lua, message.as_ptr());
                        return 1;
                    }
                },
                None => {
                    let message = CString::new("the module cannot run on this platform").unwrap();
                    lua_pushstring(lua, message.as_ptr());
                    return 1;
                }
            },
        };

        // Add module to loaded table.
        // TODO: Use try_insert once https://github.com/rust-lang/rust/issues/82766 is stable.
        if loaded.insert(module, native).is_some() {
            panic!("Some module was loaded twice somehow")
        }

        2
    }

    fn bootstrap_native_module(
        context: *const api::BootstrapContext,
        file: &Path,
    ) -> Option<Library> {
        // Load the module.
        let library = match unsafe { Library::new(&file) } {
            Ok(r) => r,
            Err(e) => {
                let message = cfmt!("cannot load {}: {}", file.display(), e);
                unsafe { lua_pushstring((*context).lua, message.as_ptr()) };
                return None;
            }
        };

        // Get bootstrap function.
        let bootstrap = match unsafe { library.get::<ModuleBootstrap>(b"bootstrap\0") } {
            Ok(r) => r,
            Err(e) => {
                let message = cfmt!(
                    "cannot find bootstrap function in {}: {}",
                    file.display(),
                    e
                );
                unsafe { lua_pushstring((*context).lua, message.as_ptr()) };
                return None;
            }
        };

        // Bootstrap the module.
        match unsafe { bootstrap(context, &api::TABLE) } {
            1 => None,
            2 => Some(library),
            _ => panic!(
                "{} return an unexpected value from bootstrap function",
                file.display()
            ),
        }
    }

    fn open_library(&mut self, name: &[u8], function: LuaFunction) {
        if let Some(b) = name.last() {
            if *b != 0 {
                panic!("Name must be null-terminated")
            }
        } else {
            panic!("Name cannot be empty")
        }

        unsafe { luaL_requiref(self.lua, transmute(name.as_ptr()), Some(function), 1) }
    }

    fn get_field(&self, index: c_int, key: &str) -> c_int {
        let key = CString::new(key).unwrap();

        unsafe { lua_getfield(self.lua, index, key.as_ptr()) }
    }

    fn set_index(&mut self, table: c_int, index: lua_Integer) {
        unsafe { lua_seti(self.lua, table, index) };
    }

    fn to_string(&mut self, index: c_int) -> Option<String> {
        let value = unsafe { lua_tolstring(self.lua, index, null_mut()) };

        if value.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(value).to_str().unwrap().into() })
        }
    }

    fn push_nil(&mut self) {
        unsafe { lua_pushnil(self.lua) };
    }

    fn push_fn(&mut self, function: LuaFunction, up_count: c_int) {
        unsafe { lua_pushcclosure(self.lua, Some(function), up_count) };
    }

    fn push_light_userdata(&mut self, data: *mut c_void) {
        unsafe { lua_pushlightuserdata(self.lua, data) }
    }

    fn pop_string(&mut self) -> Option<String> {
        if let Some(v) = self.to_string(-1) {
            self.pop_stack(1);
            Some(v)
        } else {
            None
        }
    }

    fn pop_stack(&mut self, count: c_int) {
        unsafe { lua_settop(self.lua, -count - 1) };
    }
}

impl<'context> Drop for Engine<'context> {
    fn drop(&mut self) {
        if !self.lua.is_null() {
            unsafe { lua_close(self.lua) };
        }
    }
}

/// Represents the error from execution of a Lua script.
pub enum RunError<A> {
    LoadError(String),
    ArgumentError(A),
    ExecError(String),
}

/// A trait to convert Rust value(s) to Lua value(s).
pub trait ToLua {
    type Err;

    fn to_lua(&self, lua: *mut lua_State) -> Result<c_int, Self::Err>;
}

impl<T> ToLua for T
where
    T: AsRef<str>,
{
    type Err = std::ffi::NulError;

    fn to_lua(&self, lua: *mut lua_State) -> Result<c_int, Self::Err> {
        let v = CString::new(self.as_ref())?;
        unsafe { lua_pushstring(lua, v.as_ptr()) };
        Ok(1)
    }
}

type ModuleTable<'context> = HashMap<Module<'context, 'static>, Option<Library>>;
type LuaFunction = unsafe extern "C" fn(*mut lua_State) -> c_int;
type ModuleBootstrap =
    unsafe extern "C" fn(*const api::BootstrapContext, *const api::Table) -> c_int;
