use crate::Dispatcher;
use std::collections::HashMap;
use std::ptr::null_mut;
use std::task::Waker;

#[cfg(target_family = "unix")]
pub type Socket = std::os::unix::io::RawFd;
#[cfg(target_family = "windows")]
pub type Socket = std::os::windows::io::RawSocket;

pub static mut DISPATCHER: Option<*mut dyn Dispatcher> = None;
pub static mut PENDING: *mut HashMap<Socket, PendingData> = null_mut();

pub struct PendingData {
    pub waker: Waker,
    pub cancelable: bool,
}
