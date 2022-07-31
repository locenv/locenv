use crate::ffi::{
    kami_winsock_event_dispatch, kami_winsock_event_init, kami_winsock_event_watch_accept,
    kami_winsock_event_watch_read, kami_winsock_event_watch_remove, kami_winsock_event_watch_write,
};
use crate::state::Socket;
use crate::Dispatcher;
use std::ffi::c_void;
use std::mem::transmute;

/// Represents a [`Dispatcher`] using `WSAWaitForMultipleEvents` on Windows.
///
/// This dispatcher supports a very limited concurrent connections. It was designed for a specific
/// use case that don't need to handle many connections.
pub struct WinsockEvent {
    interrupt_handler: Option<Box<dyn FnMut() -> bool>>,
}

impl WinsockEvent {
    /// Create a dispatcher.
    ///
    /// # Panics
    ///
    /// This function can be called only once. It will panic on the second call.
    pub fn new() -> Self {
        if unsafe { kami_winsock_event_init() } != 0 {
            panic!("WinsockEvent cannot create more than one");
        }

        Self {
            interrupt_handler: None,
        }
    }

    /// Sets a closure to be called when `WSAWaitForMultipleEvents` has been interupted by I/O
    /// completion routine.
    ///
    /// The return value of the closure indicated whether to continue on dispatching events or stop
    /// and return from [`crate::block_on`].
    pub fn set_interrupt_handler<H: FnMut() -> bool + 'static>(&mut self, handler: H) {
        self.interrupt_handler = Some(Box::new(handler));
    }
}

impl Dispatcher for WinsockEvent {
    fn watch_for_accept(&mut self, socket: Socket) {
        let r = unsafe { kami_winsock_event_watch_accept(socket) };

        if r < 0 {
            panic!(
                "Winsock error while trying to listen for accept notification ({})",
                r.abs()
            );
        }

        match r {
            0 => {}
            1 => panic!("Maximum number of sockets has been reached"),
            _ => panic!(
                "Got an unexpected result from kami_winsock_event_watch_accept: {}",
                r
            ),
        }
    }

    fn watch_for_read(&mut self, socket: Socket) {
        let r = unsafe { kami_winsock_event_watch_read(socket) };

        if r < 0 {
            panic!(
                "Winsock error while trying to listen for read notification ({})",
                r.abs()
            );
        }

        match r {
            0 => {}
            1 => panic!("Maximum number of sockets has been reached"),
            _ => panic!(
                "Got an unexpected result from kami_winsock_event_watch_read: {}",
                r
            ),
        }
    }

    fn watch_for_write(&mut self, socket: Socket) {
        let r = unsafe { kami_winsock_event_watch_write(socket) };

        if r < 0 {
            panic!(
                "Winsock error while trying to listen for write notification ({})",
                r.abs()
            );
        }

        match r {
            0 => {}
            1 => panic!("Maximum number of sockets has been reached"),
            _ => panic!(
                "Got an unexpected result from kami_winsock_event_watch_write: {}",
                r
            ),
        }
    }

    fn remove_watch(&mut self, socket: Socket) {
        let r = unsafe { kami_winsock_event_watch_remove(socket) };

        if r < 0 {
            panic!(
                "Winsock error while trying to stop listening for notification ({})",
                r.abs()
            );
        }

        match r {
            0 | 1 => {}
            _ => panic!(
                "Got an unexpected result from kami_winsock_event_watch_remove: {}",
                r
            ),
        }
    }

    fn run<H: FnMut(Socket)>(&mut self, mut handler: H) -> bool
    where
        Self: Sized,
    {
        loop {
            let result =
                unsafe { kami_winsock_event_dispatch(ready::<H>, transmute(&mut handler)) };

            if result < 0 {
                panic!("Winsock error while waiting for events ({})", result.abs());
            }

            match result {
                0 => return true,
                1 => {
                    panic!("No socket to watch, some future implementations forgot to register it")
                }
                2 => {
                    if let Some(h) = self.interrupt_handler.as_mut() {
                        if !h() {
                            return false;
                        }
                    }
                }
                v => panic!(
                    "Got an unexpected result from kami_winsock_event_dispatch ({})",
                    v
                ),
            }
        }

        unsafe extern "C" fn ready<H: FnMut(Socket)>(socket: Socket, context: *mut c_void) {
            let handler: *mut H = transmute(context);

            (*handler)(socket);
        }
    }
}
