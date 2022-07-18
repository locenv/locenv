use crate::SUCCESS;
use context::Context;
use dirtree::TempFile;
use std::ffi::CString;
use std::net::{SocketAddr, TcpListener};

pub const INITIALIZATION_FAILED: u8 = 254;

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

    let server = TcpListener::bind("127.0.0.1:0").unwrap();

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

    // TODO: Report status to the parent.

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
