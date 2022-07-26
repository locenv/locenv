use super::DISPATCHER_TERMINATED;
use crate::SUCCESS;
use std::collections::{HashMap, HashSet, LinkedList};
use std::future::Future;
use std::mem::transmute;
use std::net::{SocketAddr, TcpListener, TcpStream};
#[cfg(target_family = "unix")]
use std::os::unix::prelude::AsRawFd;
#[cfg(target_family = "windows")]
use std::os::windows::io::AsRawSocket;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

pub fn accept<'a>(listener: &'a TcpListener) -> Accepting<'a> {
    Accepting(listener)
}

pub fn spawn(task: impl Future<Output = ()> + 'static) {
    let task = Box::pin(task);

    unsafe { READY.as_mut().unwrap().push_back(task) };
}

pub fn run(task: impl Future<Output = ()> + 'static) -> u8 {
    let task = Box::pin(task);

    // Initialize global tables.
    unsafe {
        if PENDING.is_some() || READY.is_some() || WAKERS.is_some() {
            panic!("Dispatcher cannot run more than one");
        }

        PENDING = Some(HashSet::new());
        READY = Some(LinkedList::new());
        WAKERS = Some(HashMap::new());
    }

    // Enter event dispatching loop.
    unsafe { READY.as_mut().unwrap().push_back(task) };

    let result = loop {
        // Poll all tasks that are ready.
        while let Some(t) = unsafe { READY.as_mut().unwrap().pop_front() } {
            let data = Rc::new(WakerData { task: Some(t) });
            let data = Rc::into_raw(data) as *mut WakerData;
            let waker = unsafe { Waker::from_raw(RawWaker::new(transmute(data), &RAW_WAKER)) };
            let mut context = Context::from_waker(&waker);

            match unsafe { (*data).task.as_mut().unwrap().as_mut().poll(&mut context) } {
                Poll::Ready(_) => unsafe {
                    Rc::from_raw(data);
                },
                Poll::Pending => unsafe {
                    PENDING.as_mut().unwrap().insert(transmute(data));
                },
            }
        }

        // Check if no pending tasks.
        unsafe {
            assert_eq!(
                PENDING.as_ref().unwrap().len(),
                WAKERS.as_ref().unwrap().len()
            );

            if PENDING.as_ref().unwrap().is_empty() {
                break SUCCESS;
            }
        }

        // Wait for events.
        let result = unsafe { dispatch_events(handle_event) };

        if result == DISPATCHER_TERMINATED {
            break SUCCESS;
        } else if result != SUCCESS {
            break result;
        }
    };

    // Destroy global tables.
    unsafe {
        READY = None;
        WAKERS = None;

        for data in PENDING.take().unwrap() {
            Rc::from_raw(data as *const WakerData);
        }
    }

    result
}

pub struct Accepting<'a>(&'a TcpListener);

impl<'a> Future for Accepting<'a> {
    type Output = Result<(TcpStream, SocketAddr), std::io::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        // Accept the connection.
        match self.0.accept() {
            Ok(r) => return Poll::Ready(Ok(r)),
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    return Poll::Ready(Err(e));
                }
            }
        }

        // Register for notification.
        #[cfg(target_family = "unix")]
        let socket = self.0.as_raw_fd();
        #[cfg(target_family = "windows")]
        let socket = self.0.as_raw_socket();

        unsafe { register_for_accept(socket) };

        match unsafe { WAKERS.as_mut().unwrap().insert(socket, cx.waker().clone()) } {
            Some(_) => panic!("Accept cannot be called while there are any pending"),
            None => Poll::Pending,
        }
    }
}

static mut PENDING: Option<HashSet<usize>> = None;
static mut READY: Option<LinkedList<Pin<Box<dyn Future<Output = ()>>>>> = None;
static mut WAKERS: Option<HashMap<Socket, Waker>> = None;

const RAW_WAKER: RawWakerVTable =
    RawWakerVTable::new(waker_clone, waker_wake, waker_wake_ref, waker_drop);

unsafe extern "C" fn handle_event(socket: Socket) {
    let wakers = WAKERS.as_mut().unwrap();
    let waker = wakers.remove(&socket).unwrap();

    waker.wake();
}

unsafe fn waker_clone(data: *const ()) -> RawWaker {
    let data = Rc::from_raw(data as *const WakerData);
    let cloned = data.clone();

    Rc::into_raw(data);

    RawWaker::new(transmute(Rc::into_raw(cloned)), &RAW_WAKER)
}

unsafe fn waker_wake(data: *const ()) {
    // Remove from pending.
    PENDING.as_mut().unwrap().remove(&transmute(data));
    Rc::from_raw(data as *const WakerData);

    // Move to ready.
    let data: *mut WakerData = transmute(data);

    READY
        .as_mut()
        .unwrap()
        .push_back((*data).task.take().unwrap());

    Rc::from_raw(data);
}

unsafe fn waker_wake_ref(data: *const ()) {
    // Remove from pending.
    PENDING.as_mut().unwrap().remove(&transmute(data));
    Rc::from_raw(data as *const WakerData);

    // Move to ready.
    let data: *mut WakerData = transmute(data);

    READY
        .as_mut()
        .unwrap()
        .push_back((*data).task.take().unwrap());
}

unsafe fn waker_drop(data: *const ()) {
    Rc::from_raw(data as *const WakerData);
}

struct WakerData {
    task: Option<Pin<Box<dyn Future<Output = ()>>>>,
}

#[cfg(target_family = "unix")]
type Socket = std::os::unix::io::RawFd;
#[cfg(target_family = "windows")]
type Socket = std::os::windows::io::RawSocket;

extern "C" {
    fn register_for_accept(socket: Socket);
    fn dispatch_events(handler: unsafe extern "C" fn(Socket)) -> u8;
}
