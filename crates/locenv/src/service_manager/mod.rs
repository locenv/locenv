use self::client::{Client, ConsoleConnection};
use crate::SUCCESS;
use context::Context;
use std::ffi::CString;

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

    let log = unsafe {
        let path = context
            .project()
            .runtime(true)
            .unwrap()
            .service_manager(true)
            .unwrap()
            .log();
        let path = CString::new(path.to_str().unwrap()).unwrap();

        log_stderr(path.as_ptr())
    };

    // Create a connection with the parent.
    let parent = Client::new(ConsoleConnection::new());

    // TODO:
    // - Send a response to tell that initialization is completed and port file has been written.
    // - Reopen STDOUT to log file.

    SUCCESS
}

extern "C" {
    #[cfg(target_family = "unix")]
    fn log_stderr(path: *const std::os::raw::c_char) -> std::os::raw::c_int;

    #[cfg(target_family = "windows")]
    fn log_stderr(path: *const std::os::raw::c_char) -> *const std::ffi::c_void;
}
