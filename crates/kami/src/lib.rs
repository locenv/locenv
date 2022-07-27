use self::futures::Accepting;
use self::state::{Socket, DISPATCHER, WAKERS};
use std::collections::{HashMap, LinkedList};
use std::future::Future;
use std::mem::transmute;
use std::net::TcpListener;
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

/// Block the current thread until all futures is completed.
///
/// Returns a value indicated whether all futures is running to completion or the `dispatcher` want
/// to stop.
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
    let task = Box::pin(task);

    // Initialize global tables.
    let mut wakers: HashMap<Socket, Waker> = HashMap::new();
    let mut ready: LinkedList<Pin<Box<dyn Future<Output = ()>>>> = LinkedList::new();

    unsafe {
        if DISPATCHER.is_some() {
            panic!("block_on can be called only once");
        }

        DISPATCHER = Some(transmute(&*dispatcher as &dyn Dispatcher));
        READY = transmute(&ready);
        WAKERS = transmute(&wakers);
    }

    // Enter event dispatching loop.
    ready.push_back(task);

    loop {
        // Poll all tasks that are ready.
        while let Some(t) = ready.pop_front() {
            let data = Rc::new(WakerData { task: Some(t) });
            let data = Rc::into_raw(data) as *mut WakerData;
            let waker = unsafe { Waker::from_raw(RawWaker::new(transmute(data), &RAW_WAKER)) };
            let mut context = Context::from_waker(&waker);

            // #[allow(unused_must_use)] does not work somehow...
            let _ = unsafe { (*data).task.as_mut().unwrap().as_mut().poll(&mut context) };
        }

        // Check if no pending tasks.
        if wakers.is_empty() {
            break true;
        }

        // Wait for events.
        if !dispatcher.run(|s| wakers.remove(&s).unwrap().wake()) {
            assert!(ready.is_empty());
            break false;
        }
    }
}

/// Represents the dispatcher to dispatch the I/O events.
pub trait Dispatcher {
    fn watch_for_accept(&mut self, socket: Socket);

    fn run<H: FnMut(Socket)>(&mut self, handler: H) -> bool
    where
        Self: Sized;
}

struct WakerData {
    task: Option<Pin<Box<dyn Future<Output = ()>>>>,
}

static mut READY: *mut LinkedList<Pin<Box<dyn Future<Output = ()>>>> = null_mut();

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
