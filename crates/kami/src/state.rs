use crate::Dispatcher;
use std::collections::HashMap;
use std::ptr::null_mut;
use std::task::Waker;

#[cfg(target_family = "unix")]
pub type Socket = std::os::unix::io::RawFd;
#[cfg(target_family = "windows")]
pub type Socket = std::os::windows::io::RawSocket;

pub static mut DISPATCHER: Option<*mut dyn Dispatcher> = None;
pub static mut WAKERS: *mut HashMap<Socket, Waker> = null_mut();
