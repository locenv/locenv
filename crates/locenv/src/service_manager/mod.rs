use self::responses::ServiceManagerStatus;
use context::Context;
use dirtree::TempFile;
use std::ffi::CString;
use std::net::{SocketAddr, TcpListener};
#[cfg(target_family = "unix")]
use std::os::unix::prelude::AsRawFd;
#[cfg(target_family = "windows")]
use std::os::windows::io::AsRawSocket;

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
        .runtime(true)
        .unwrap()
        .service_manager(true)
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
        client::Request::GetStatus => {
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

    // Enter event loop
    #[cfg(target_family = "unix")]
    unsafe {
        event_loop(server.as_raw_fd())
    }

    #[cfg(target_family = "windows")]
    unsafe {
        event_loop(server.as_raw_socket())
    }
}

extern "C" {
    #[cfg(target_family = "unix")]
    fn redirect_console_output(file: *const std::os::raw::c_char);

    #[cfg(target_family = "windows")]
    fn redirect_console_output(file: *const std::os::raw::c_char);

    #[cfg(target_family = "unix")]
    fn event_loop(server: std::os::raw::c_int) -> u8;

    #[cfg(target_family = "windows")]
    fn event_loop(server: std::os::windows::raw::SOCKET) -> u8;
}
