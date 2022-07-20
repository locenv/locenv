use self::responses::ServiceManagerStatus;
use crate::SUCCESS;
use context::Context;
use dirtree::TempFile;
use std::ffi::CString;
use std::net::{SocketAddr, TcpListener};

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

    SUCCESS
}

extern "C" {
    #[cfg(target_family = "unix")]
    fn redirect_console_output(file: *const std::os::raw::c_char);

    #[cfg(target_family = "windows")]
    fn redirect_console_output(file: *const std::os::raw::c_char);
}
