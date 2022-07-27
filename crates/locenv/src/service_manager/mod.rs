use crate::SUCCESS;
use context::Context;
use dirtree::TempFile;
use std::ffi::{c_void, CString};
use std::io::Write;
use std::mem::transmute;
use std::net::{SocketAddr, TcpListener};
use std::os::raw::{c_char, c_int};
use std::path::PathBuf;

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
    print!("locenv-ok");
    std::io::stdout().flush().unwrap();

    // Enter background.
    let log = context
        .project()
        .runtime(false)
        .unwrap()
        .service_manager(false)
        .unwrap()
        .log();

    let data = DaemonData {
        context: Some(context),
        server: Some(server),
    };

    daemon(log, data)
}

async fn main(data: &mut DaemonData) {
    let context = data.context.take().unwrap();
    let server = data.server.take().unwrap();

    // Write PID file.
    let pid = context
        .project()
        .runtime(false)
        .unwrap()
        .service_manager(false)
        .unwrap()
        .pid();
    let pid = TempFile::new(pid);

    pid.write(&std::process::id()).unwrap();

    // Enter main loop.
    loop {
        // Accept a connection from RPC client
        let (client, from) = match kami::accept(&server).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to accept a connection from RPC client: {}", e);
                continue;
            }
        };
    }
}

fn daemon(log: PathBuf, mut data: DaemonData) -> u8 {
    let log = CString::new(log.to_str().unwrap()).unwrap();

    unsafe { enter_daemon(log.as_ptr(), daemon_procedure, transmute(&mut data)) }
}

#[cfg(target_family = "unix")]
fn create_dispatcher() -> kami::dispatcher::unix::Pselect {
    let mut d = kami::dispatcher::unix::Pselect::new();

    d.allow_signal(15); // SIGTERM
    d.set_interrupt_handler(|| unsafe { is_shutdown_requested() } == 0);

    d
}

#[cfg(target_family = "windows")]
fn create_dispatcher() -> kami::dispatcher::win32::WinsockEvent {
    let mut d = kami::dispatcher::win32::WinsockEvent::new();

    d.set_interrupt_handler(|| unsafe { is_shutdown_requested() } == 0);

    d
}

unsafe extern "C" fn daemon_procedure(context: *mut c_void) -> u8 {
    let data: *mut DaemonData = transmute(context);

    kami::block_on(create_dispatcher(), main(&mut *data));

    SUCCESS
}

struct DaemonData {
    context: Option<Context>,
    server: Option<TcpListener>,
}

type DaemonProcedure = unsafe extern "C" fn(*mut c_void) -> u8;

extern "C" {
    fn is_shutdown_requested() -> c_int;
    fn enter_daemon(log: *const c_char, daemon: DaemonProcedure, context: *mut c_void) -> u8;
}
