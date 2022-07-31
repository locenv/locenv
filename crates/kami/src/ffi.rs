use crate::state::Socket;
use std::ffi::c_void;
use std::os::raw::c_int;

#[cfg(target_family = "unix")]
#[repr(C)]
pub struct DispatchHandlers {
    pub interrupted: unsafe extern "C" fn(*mut c_void) -> c_int,
    pub ready: unsafe extern "C" fn(Socket, *mut c_void),
}

#[cfg(target_family = "unix")]
extern "C" {
    pub fn kami_pselect_init() -> c_int;
    pub fn kami_pselect_watch_read(socket: Socket);
    pub fn kami_pselect_watch_write(socket: Socket);
    pub fn kami_pselect_watch_remove(socket: Socket);
    pub fn kami_pselect_dispatch(
        signals: *const c_int,
        signals_count: c_int,
        handlers: *const DispatchHandlers,
        context: *mut c_void,
    ) -> c_int;
}

#[cfg(target_family = "windows")]
extern "C" {
    pub fn kami_winsock_event_init() -> c_int;
    pub fn kami_winsock_event_watch_accept(socket: Socket) -> c_int;
    pub fn kami_winsock_event_watch_read(socket: Socket) -> c_int;
    pub fn kami_winsock_event_watch_write(socket: Socket) -> c_int;
    pub fn kami_winsock_event_watch_remove(socket: Socket) -> c_int;
    pub fn kami_winsock_event_dispatch(
        handler: unsafe extern "C" fn(Socket, *mut c_void),
        context: *mut c_void,
    ) -> c_int;
}
