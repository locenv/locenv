#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use libc::{c_char, c_int, intptr_t, size_t};
use std::ffi::CStr;

// Lua API.

pub const LUA_OK: c_int = 0;

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
    pub fn lua_tolstring(L: *mut lua_State, idx: c_int, len: *mut size_t) -> *const c_char;
    pub fn lua_pcallk(
        L: *mut lua_State,
        nargs: c_int,
        nresults: c_int,
        errfunc: c_int,
        ctx: lua_KContext,
        k: Option<lua_KFunction>,
    ) -> c_int;
    pub fn lua_settop(L: *mut lua_State, idx: c_int);

    // Standard libraries.
    pub fn luaopen_base(L: *mut lua_State) -> c_int;

    // Auxiliary library.
    pub fn luaL_newstate() -> *mut lua_State;
    pub fn luaL_loadstring(L: *mut lua_State, s: *const c_char) -> c_int;
}

// Helper.

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
