#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub use std::os::raw::c_int;

use std::ffi::{CStr, CString};
use std::mem::size_of;

include!(concat!(env!("OUT_DIR"), "/lua.rs"));

// Helper.

pub fn check_string(L: *mut lua_State, arg: c_int) -> Option<String> {
    let v = unsafe { luaL_checklstring(L, arg, std::ptr::null_mut()) };

    if v.is_null() {
        return None;
    }

    Some(unsafe { CStr::from_ptr(v).to_str().unwrap().into() })
}

pub fn argument_error(L: *mut lua_State, arg: c_int, msg: &str) -> ! {
    let c = CString::new(msg).unwrap();

    unsafe { luaL_argerror(L, arg, c.as_ptr()) };

    // This should never happen.
    panic!();
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

pub fn push_closure<C: FnMut(*mut lua_State) -> c_int>(L: *mut lua_State, r#fn: C) {
    // Push the closure.
    let boxed = Box::into_raw(Box::new(r#fn));
    let up = unsafe { lua_newuserdatauv(L, size_of::<*mut C>() as _, 1) };

    unsafe { std::ptr::copy_nonoverlapping(&boxed as *const _ as *const u8, up as *mut u8, size_of::<*mut C>()) };

    // Associate the closure with metatable.
    let table = CString::new("locenv.closure").unwrap();

    if unsafe { luaL_newmetatable(L, table.as_ptr()) } == 1 {
        push_string(L, "__gc");
        unsafe { lua_pushcclosure(L, Some(free_closure::<C>), 0) };
        unsafe { lua_settable(L, -3) };
    }

    unsafe { lua_setmetatable(L, -2) };

    // Push the executor.
    unsafe { lua_pushcclosure(L, Some(execute_closure::<C>), 1) };
}

pub fn get_field(L: *mut lua_State, idx: c_int, k: &str) -> c_int {
    let c = CString::new(k).unwrap();

    unsafe { lua_getfield(L, idx, c.as_ptr()) }
}

pub fn set_field(L: *mut lua_State, idx: c_int, k: &str) {
    let c = CString::new(k).unwrap();

    unsafe { lua_setfield(L, idx, c.as_ptr()) };
}

extern "C" fn execute_closure<C: FnMut(*mut lua_State) -> c_int>(L: *mut lua_State) -> c_int {
    let up = unsafe { lua_touserdata(L, LUA_REGISTRYINDEX - 1) };
    let boxed: *mut C = std::ptr::null_mut();

    unsafe { std::ptr::copy_nonoverlapping(up as *mut u8, &boxed as *const _ as *mut u8, size_of::<*mut C>()) };

    unsafe { (*boxed)(L) }
}

extern "C" fn free_closure<C: FnMut(*mut lua_State) -> c_int>(L: *mut lua_State) -> c_int {
    // Get the closure.
    let table = CString::new("locenv.closure").unwrap();
    let closure = unsafe { luaL_checkudata(L, 1, table.as_ptr()) };

    if closure.is_null() {
        argument_error(L, 1, "`internal closure' expected");
    }

    // Convert to Rust object.
    let boxed: *mut C = std::ptr::null_mut();

    unsafe { std::ptr::copy_nonoverlapping(closure as *mut u8, &boxed as *const _ as *mut u8, size_of::<*mut C>()) };

    // Destroy the closure.
    unsafe { Box::from_raw(boxed) };

    0
}
