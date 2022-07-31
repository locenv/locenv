use self::futures::{Accepting, Reading, Writing};
use self::state::{PendingData, Socket, DISPATCHER, PENDING};
use std::collections::{HashMap, LinkedList};
use std::future::Future;
use std::mem::transmute;
use std::net::{TcpListener, TcpStream};
use std::pin::Pin;
use std::ptr::null_mut;
use std::rc::Rc;
use std::task::{Context, RawWaker, RawWakerVTable, Waker};

pub mod dispatcher;
pub mod futures;

mod ffi;
mod state;

/// Asynchronous version of [`std::net::TcpListener::accept()`]. The `listener` **MUST** be in
/// non-blocking mode otherwise the call to this function will be block.
///
/// # Panics
///
/// The returned future must be polling inside Kami runtime (inside the future that passed in
/// [`block_on`]) otherwise it will panic.
pub fn accept<'a>(listener: &'a TcpListener) -> Accepting<'a> {
    Accepting::new(listener)
}

/// Asynchronous version of [`TcpStream::read`]. The `stream` **MUST** be in non-blocking mode
/// otherwise the call to this function will be block.
pub fn read<'a, 'b>(stream: &'a mut TcpStream, buf: &'b mut [u8]) -> Reading<'a, 'b> {
    Reading::new(stream, buf)
}

/// Asynchronous version of [`TcpStream::write`]. The `stream` **MUST** be in non-blocking mode
/// otherwise the call to this function may block.
pub fn write<'a, 'b>(stream: &'a mut TcpStream, buf: &'b [u8]) -> Writing<'a, 'b> {
    Writing::new(stream, buf)
}

/// Start a new [`Future`] to run concurrently with the other [`Future`].
///
/// # Panics
///
/// This function will panic if it does not called inside a future chain that currently running by
/// [`block_on`].
///
/// # Examples
///
/// ```no_run
/// kami::spawn(async {
///     println!("Hello from a concurrent Future!");
/// });
/// ```
pub fn spawn(task: impl Future<Output = ()> + 'static) {
    let task = Box::pin(task);

    if unsafe { READY.is_null() } {
        panic!("spawn cannot be called outside the Kami runtime");
    }

    unsafe { (*READY).push_back(task) };
}

/// Trigger graceful shutdown. This function will return immediately.
pub fn shutdown() {
    unsafe { SHUTDOWN = true };
}

/// Block the current thread until all futures is completed.
///
/// Returns a value indicated whether all futures is running to completion or shutdown has been
/// triggered by either `dispatcher` or [`shutdown`].
///
/// # Panics
///
/// This function can be called only once. It will panic on the second call.
pub fn block_on<D, T>(dispatcher: D, task: T) -> bool
where
    D: Dispatcher + 'static,
    T: Future<Output = ()> + 'static,
{
    let mut dispatcher = Box::new(dispatcher);

    // Initialize global tables.
    let mut pending: HashMap<Socket, PendingData> = HashMap::new();
    let mut ready: LinkedList<Pin<Box<dyn Future<Output = ()>>>> = LinkedList::new();

    unsafe {
        if DISPATCHER.is_some() {
            panic!("block_on can be called only once");
        }

        DISPATCHER = Some(transmute(&*dispatcher as &dyn Dispatcher));
        READY = transmute(&ready);
        PENDING = transmute(&pending);
    }

    // Enter event dispatching loop.
    ready.push_back(Box::pin(task));

    let result = loop {
        // Poll all tasks that are ready.
        poll_all(&mut ready);

        // Shoud we stop the event loop?
        if pending.is_empty() {
            break true;
        } else if unsafe { SHUTDOWN } {
            break false;
        }

        // Wait for events.
        if !dispatcher.run(|s| pending.remove(&s).unwrap().waker.wake()) {
            break false;
        }
    };

    assert!(ready.is_empty());

    // Wait all non-cancelable futures.
    while !pending.is_empty() {
        // Remove all cancelable tasks from the above loop and newly added by the end of this loop
        // so we don't end up running this as a main loop.
        pending.retain(|s, d| {
            if d.cancelable {
                dispatcher.remove_watch(*s);
                false
            } else {
                true
            }
        });

        if pending.is_empty() {
            break;
        }

        // Wait some of non-cancelable to be ready and poll it.
        dispatcher.run(|s| pending.remove(&s).unwrap().waker.wake());

        poll_all(&mut ready);
    }

    result
}

fn poll_all(tasks: &mut LinkedList<Pin<Box<dyn Future<Output = ()>>>>) {
    while let Some(t) = tasks.pop_front() {
        let data = Rc::new(WakerData { task: Some(t) });
        let data = Rc::into_raw(data) as *mut WakerData;
        let waker = unsafe { Waker::from_raw(RawWaker::new(transmute(data), &RAW_WAKER)) };
        let mut context = Context::from_waker(&waker);

        // #[allow(unused_must_use)] does not work somehow...
        let _ = unsafe { (*data).task.as_mut().unwrap().as_mut().poll(&mut context) };
    }
}

/// Represents the dispatcher to dispatch the I/O events.
pub trait Dispatcher {
    fn watch_for_accept(&mut self, socket: Socket);
    fn watch_for_read(&mut self, socket: Socket);
    fn watch_for_write(&mut self, socket: Socket);
    fn remove_watch(&mut self, socket: Socket);

    fn run<H: FnMut(Socket)>(&mut self, handler: H) -> bool
    where
        Self: Sized;
}

struct WakerData {
    task: Option<Pin<Box<dyn Future<Output = ()>>>>,
}

static mut READY: *mut LinkedList<Pin<Box<dyn Future<Output = ()>>>> = null_mut();
static mut SHUTDOWN: bool = false;

const RAW_WAKER: RawWakerVTable =
    RawWakerVTable::new(waker_clone, waker_wake, waker_wake_ref, waker_drop);

unsafe fn waker_clone(data: *const ()) -> RawWaker {
    let data = Rc::from_raw(data as *const WakerData);
    let cloned = data.clone();
    Rc::into_raw(data);

    RawWaker::new(transmute(Rc::into_raw(cloned)), &RAW_WAKER)
}

unsafe fn waker_wake(data: *const ()) {
    let data: *mut WakerData = transmute(data);

    (*READY).push_back((*data).task.take().unwrap());

    Rc::from_raw(data);
}

unsafe fn waker_wake_ref(data: *const ()) {
    let data: *mut WakerData = transmute(data);

    (*READY).push_back((*data).task.take().unwrap());
}

unsafe fn waker_drop(data: *const ()) {
    Rc::from_raw(data as *const WakerData);
}
