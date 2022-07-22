use self::client::RequestData;
use self::responses::ServiceManagerStatus;
use crate::SUCCESS;
use context::Context;
use dirtree::TempFile;
use std::ffi::{c_void, CString};
use std::mem::transmute;
use std::net::{SocketAddr, TcpListener};
#[cfg(target_family = "unix")]
use std::os::unix::prelude::AsRawFd;
#[cfg(target_family = "windows")]
use std::os::windows::io::AsRawSocket;

#[no_mangle]
pub static SIGACTION_FAILED: u8 = 238;
pub const INVALID_SERVER_SOCKET: u8 = 239;
pub const UNKNOW_EVENT: u8 = 240;
#[no_mangle]
pub static WAIT_CLIENT_FAILED: u8 = 241;
#[no_mangle]
pub static GET_WINDOW_LONG_FAILED: u8 = 242;
#[no_mangle]
pub static SET_WINDOW_LONG_FAILED: u8 = 243;
#[no_mangle]
pub static ASYNC_SELECT_FAILED: u8 = 244;
#[no_mangle]
pub static BLOCK_SIGNALS_FAILED: u8 = 245;
#[no_mangle]
pub static SELECT_FAILED: u8 = 246;
#[no_mangle]
pub static CREATE_WINDOW_FAILED: u8 = 247;
#[no_mangle]
pub static REGISTER_CLASS_FAILED: u8 = 248;
#[no_mangle]
pub static EVENT_LOOP_FAILED: u8 = 249;
pub const SEND_PARENT_RESPONSE_FAILED: u8 = 250;
pub const INVALID_PARENT_REQUEST: u8 = 251;
pub const READ_PARENT_REQUEST_FAILED: u8 = 252;
pub const START_RPC_SERVER_FAILED: u8 = 253;
pub const INITIALIZATION_FAILED: u8 = 254;

pub mod requests;
pub mod responses;

mod client;

pub fn run() -> u8 {
    // Initialize foundation.
    let context = match Context::new(std::env::current_dir().unwrap()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e);
            return INITIALIZATION_FAILED;
        }
    };

    let pid = context
        .project()
        .runtime(true)
        .unwrap()
        .service_manager(true)
        .unwrap()
        .pid();
    let pid = TempFile::new(pid);

    pid.write(&std::process::id()).unwrap();

    // Start RPC server.
    let server = match TcpListener::bind("127.0.0.1:0") {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to start RPC server on 127.0.0.1: {}", e);
            return START_RPC_SERVER_FAILED;
        }
    };

    server.set_nonblocking(true).unwrap();

    // Write port file.
    let port = context
        .project()
        .runtime(false)
        .unwrap()
        .service_manager(false)
        .unwrap()
        .port();
    let port = TempFile::new(port);

    port.write(&match server.local_addr().unwrap() {
        SocketAddr::V4(a) => a.port(),
        SocketAddr::V6(a) => a.port(),
    })
    .unwrap();

    // Report status to the parent.
    let mut parent = client::Client::new(client::ConsoleConnection::new());
    let request = match parent.receive() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to read the request from the parent: {}", e);
            return READ_PARENT_REQUEST_FAILED;
        }
    };

    match request {
        RequestData::GetStatus => {
            if let Err(e) = parent.send(ServiceManagerStatus::new()) {
                eprintln!("Failed to response current status to the parent: {}", e);
                return SEND_PARENT_RESPONSE_FAILED;
            }
        }
        r => {
            eprintln!("Found an unexpected request from the parent: {:?}", r);
            return INVALID_PARENT_REQUEST;
        }
    }

    drop(parent);

    // Redirect STDOUT and STDERR to log file.
    unsafe {
        let file = context
            .project()
            .runtime(false)
            .unwrap()
            .service_manager(false)
            .unwrap()
            .log();
        let file = CString::new(file.to_str().unwrap()).unwrap();

        redirect_console_output(file.as_ptr());
    }

    // Enter event loop.
    #[cfg(target_family = "unix")]
    let socket = server.as_raw_fd();
    #[cfg(target_family = "windows")]
    let socket = server.as_raw_socket();
    let handler = EventHandler { server };

    unsafe { event_loop(socket, process_event, transmute(&handler)) }
}

struct EventHandler {
    server: TcpListener,
}

impl EventHandler {
    fn handle_client_connect(&mut self, data: *const c_void) -> u8 {
        // Check if the triggered socket is the same as what we have.
        #[cfg(target_family = "unix")]
        let valid = {
            let socket = data as std::os::unix::io::RawFd;

            socket == self.server.as_raw_fd()
        };

        #[cfg(target_family = "windows")]
        let valid = {
            let socket = data as std::os::windows::io::RawSocket;

            socket == self.server.as_raw_socket();
        };

        if !valid {
            eprintln!("Got an unexpected server socket from client connect event");
            return INVALID_SERVER_SOCKET;
        }

        SUCCESS
    }

    unsafe fn execute(&mut self, event: u32, data: *const c_void) -> u8 {
        match event {
            EVENT_CLIENT_CONNECT => self.handle_client_connect(data),
            _ => {
                eprintln!(
                    "Got an unexpected event {} from the event dispatcher",
                    event
                );
                UNKNOW_EVENT
            }
        }
    }
}

const EVENT_CLIENT_CONNECT: u32 = 0;

unsafe extern "C" fn process_event(context: *mut c_void, event: u32, data: *const c_void) -> u8 {
    let h: *mut EventHandler = transmute(context);

    (*h).execute(event, data)
}

type EventLoopHandler = unsafe extern "C" fn(*mut c_void, u32, *const c_void) -> u8;

extern "C" {
    #[cfg(target_family = "unix")]
    fn redirect_console_output(file: *const std::os::raw::c_char);

    #[cfg(target_family = "windows")]
    fn redirect_console_output(file: *const std::os::raw::c_char);

    #[cfg(target_family = "unix")]
    fn event_loop(
        server: std::os::raw::c_int,
        handler: EventLoopHandler,
        context: *mut c_void,
    ) -> u8;

    #[cfg(target_family = "windows")]
    fn event_loop(
        server: std::os::windows::raw::SOCKET,
        handler: EventLoopHandler,
        context: *mut c_void,
    ) -> u8;
}
