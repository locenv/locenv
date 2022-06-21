#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub use libc::{c_int, c_void};

use libc::{c_char, intptr_t, size_t};
use std::ffi::{CStr, CString};

// Lua types.

pub const LUA_OK: c_int = 0;
pub const LUAI_IS32INT: bool = (libc::c_uint::MAX >> 30) >= 3;
pub const LUAI_MAXSTACK: c_int = if LUAI_IS32INT { 1000000 } else { 15000 };
pub const LUA_REGISTRYINDEX: c_int = -LUAI_MAXSTACK - 1000;

pub type lua_CFunction = unsafe extern "C" fn(L: *mut lua_State) -> c_int;
pub type lua_Integer = libc::c_longlong;
pub type lua_KContext = intptr_t;
pub type lua_KFunction =
    unsafe extern "C" fn(L: *mut lua_State, status: c_int, ctx: lua_KContext) -> c_int;

#[repr(C)]
pub struct lua_State {
    private: [u8; 0],
}

extern "C" {
    // Core functions.
    pub fn lua_close(L: *mut lua_State);
    pub fn lua_getfield(L: *mut lua_State, idx: c_int, k: *const c_char) -> c_int;
    pub fn lua_pcallk(
        L: *mut lua_State,
        nargs: c_int,
        nresults: c_int,
        errfunc: c_int,
        ctx: lua_KContext,
        k: Option<lua_KFunction>,
    ) -> c_int;
    pub fn lua_pushcclosure(L: *mut lua_State, r#fn: lua_CFunction, n: c_int);
    pub fn lua_pushlightuserdata(L: *mut lua_State, p: *mut c_void);
    pub fn lua_pushnil(L: *mut lua_State);
    pub fn lua_pushstring(L: *mut lua_State, s: *const c_char) -> *const c_char;
    pub fn lua_setfield(L: *mut lua_State, idx: c_int, k: *const c_char);
    pub fn lua_seti(L: *mut lua_State, idx: c_int, n: lua_Integer);
    pub fn lua_settop(L: *mut lua_State, idx: c_int);
    pub fn lua_tolstring(L: *mut lua_State, idx: c_int, len: *mut size_t) -> *const c_char;
    pub fn lua_topointer(L: *mut lua_State, idx: c_int) -> *mut c_void;

    // Standard libraries.
    pub fn luaopen_base(L: *mut lua_State) -> c_int;
    pub fn luaopen_package(L: *mut lua_State) -> c_int;

    // Auxiliary library.
    pub fn luaL_checklstring(L: *mut lua_State, arg: c_int, l: *mut size_t) -> *const c_char;
    pub fn luaL_loadstring(L: *mut lua_State, s: *const c_char) -> c_int;
    pub fn luaL_newstate() -> *mut lua_State;
}

// Helper.

pub fn check_string(L: *mut lua_State, arg: c_int) -> Option<String> {
    let v = unsafe { luaL_checklstring(L, arg, std::ptr::null_mut()) };

    if v.is_null() {
        return None;
    }

    Some(unsafe { CStr::from_ptr(v).to_str().unwrap().into() })
}

pub fn pop_string(L: *mut lua_State) -> Option<String> {
    // Load stack value.
    let v = unsafe { lua_tolstring(L, -1, std::ptr::null_mut()) };

    if v.is_null() {
        return None;
    }

    // Create Rust string.
    let s: String = unsafe { CStr::from_ptr(v).to_str().unwrap().into() };

    pop(L, 1);

    Some(s)
}

pub fn pop(L: *mut lua_State, n: c_int) {
    unsafe { lua_settop(L, -n - 1) };
}

pub fn push_string(L: *mut lua_State, s: &str) {
    let c = CString::new(s).unwrap();

    unsafe { lua_pushstring(L, c.as_ptr()) };
}

pub fn get_field(L: *mut lua_State, idx: c_int, k: &str) -> c_int {
    let c = CString::new(k).unwrap();

    unsafe { lua_getfield(L, idx, c.as_ptr()) }
}

pub fn set_field(L: *mut lua_State, idx: c_int, k: &str) {
    let c = CString::new(k).unwrap();

    unsafe { lua_setfield(L, idx, c.as_ptr()) };
}
