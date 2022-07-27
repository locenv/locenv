use crate::ffi::{kami_pselect_dispatch, kami_pselect_init, kami_pselect_watch_accept};
use crate::state::Socket;
use crate::Dispatcher;
use std::ffi::c_void;
use std::mem::transmute;
use std::os::raw::c_int;

/// Represents a [`Dispatcher`] using `pselect` on *nix system.
pub struct Pselect {
    allowed_signals: Vec<c_int>,
    interrupt_handler: Option<Box<dyn FnMut() -> bool>>,
}

impl Pselect {
    /// Create a dispatcher.
    ///
    /// # Panics
    ///
    /// This function can be called only once. It will panic on the second call.
    pub fn new() -> Self {
        if unsafe { kami_pselect_init() } != 0 {
            panic!("Pselect cannot create more than one");
        }

        Self {
            allowed_signals: Vec::new(),
            interrupt_handler: None,
        }
    }

    /// Adds `signal` to the list of allowed signals while waiting on `pselect`.
    pub fn allow_signal(&mut self, signal: c_int) {
        self.allowed_signals.push(signal);
    }

    /// Sets a closure to be called when `pselect` has been interupted by signal.
    ///
    /// The return value of the closure indicated whether to continue on dispatching events or stop
    /// and return from [`crate::block_on`].
    pub fn set_interrupt_handler<H: FnMut() -> bool + 'static>(&mut self, handler: H) {
        self.interrupt_handler = Some(Box::new(handler));
    }
}

impl Dispatcher for Pselect {
    fn watch_for_accept(&mut self, socket: Socket) {
        unsafe { kami_pselect_watch_accept(socket) };
    }

    fn run<H: FnMut(Socket)>(&mut self, handler: H) -> bool
    where
        Self: Sized,
    {
        // Invoke pselect.
        let signals = self.allowed_signals.as_ptr();
        let signals_count = self.allowed_signals.len() as c_int;
        let handlers = crate::ffi::DispatchHandlers {
            interrupted: interrupted::<H>,
            ready: ready::<H>,
        };

        let mut context = PselectDispatchContext {
            dispatcher: self,
            ready: handler,
        };

        let result = unsafe {
            kami_pselect_dispatch(signals, signals_count, &handlers, transmute(&mut context))
        };

        // Handle error.
        if result < 0 {
            panic!("pselect system call failed with error {}", result.abs());
        }

        return match result {
            0 => true,
            1 => panic!(
                "No file descriptors to watch, some future implementations forgot to register it"
            ),
            2 => false,
            _ => panic!(
                "Got an unexpected value from kami_pselect_dispatch ({})",
                result
            ),
        };

        unsafe extern "C" fn interrupted<H: FnMut(Socket)>(context: *mut c_void) -> c_int {
            let context: *mut PselectDispatchContext<H> = transmute(context);

            if let Some(h) = (*context).dispatcher.interrupt_handler.as_mut() {
                if h() {
                    1
                } else {
                    0
                }
            } else {
                1
            }
        }

        unsafe extern "C" fn ready<H: FnMut(Socket)>(socket: Socket, context: *mut c_void) {
            let context: *mut PselectDispatchContext<H> = transmute(context);

            ((*context).ready)(socket)
        }
    }
}

struct PselectDispatchContext<'a, H: FnMut(Socket)> {
    dispatcher: &'a mut Pselect,
    ready: H,
}
