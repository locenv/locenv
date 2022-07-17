use crate::SUCCESS;
use context::Context;
use std::fs::create_dir_all;

pub const INITIALIZATION_FAILED: u8 = 254;

pub fn run() -> u8 {
    // Initialize foundation.
    let context = match Context::new(std::env::current_dir().unwrap()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e);
            return INITIALIZATION_FAILED;
        }
    };

    let path = context.project().runtime().service_manager().path();

    if !path.exists() {
        create_dir_all(path).unwrap();
    }

    let log = log_stderr(&context);

    // TODO:
    // - Open STDIN and read commands from CLI.
    // - Open STDOUT and write response to CLI.
    // - Send a response to tell that initialization is completed and port file has been written.
    // - Reopen STDOUT to log file.

    SUCCESS
}

#[cfg(not(target_os = "windows"))]
fn log_stderr(context: &Context) -> std::os::raw::c_int {
    use libc::{open, O_CLOEXEC, O_CREAT, O_TRUNC, O_WRONLY, S_IRGRP, S_IROTH, S_IRUSR, S_IWUSR};
    use std::ffi::CString;
    use std::os::raw::c_uint;

    let log = context.project().runtime().service_manager().log();
    let path = CString::new(log.to_str().unwrap()).unwrap();
    let flags = O_CREAT | O_WRONLY | O_TRUNC | O_CLOEXEC;
    let mode = S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH;
    let fd = unsafe { open(path.as_ptr(), flags, mode as c_uint) };

    if fd < 0 {
        panic!("Failed to open {}", log.display());
    }

    if unsafe { libc::dup2(fd, libc::STDERR_FILENO) } < 0 {
        panic!("Failed to overwrite STDERR")
    }

    fd
}

// https://stackoverflow.com/a/54096218/1829232
#[cfg(target_os = "windows")]
fn log_stderr(context: &Context) -> windows::Win32::Foundation::HANDLE {
    use windows::core::*;
    use windows::Win32::Foundation::*;
    use windows::Win32::Storage::FileSystem::*;
    use windows::Win32::System::Console::*;
    use windows::Win32::System::SystemServices::*;

    let path = context.project().runtime().service_manager().log();
    let file = unsafe {
        let name = w!(path);
        let access = GENERIC_WRITE;
        let share = FILE_SHARE_READ;
        let security = std::ptr::null();
        let creation = CREATE_ALWAYS;
        let attributes = FILE_ATTRIBUTE_NORMAL;

        CreateFileW(name, access, share, security, creation, attributes, 0).unwrap()
    };

    unsafe {
        SetStdHandle(STD_ERROR_HANDLE, file).unwrap();
    };

    file
}
