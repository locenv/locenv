use crate::state::{PendingData, Socket, DISPATCHER, PENDING};
use crate::Dispatcher;
use std::future::Future;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::ops::DerefMut;
#[cfg(target_family = "unix")]
use std::os::unix::prelude::AsRawFd;
#[cfg(target_family = "windows")]
use std::os::windows::io::AsRawSocket;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Represents a [`Future`] for asynchronous [`TcpListener::accept()`].
pub struct Accepting<'a> {
    listener: &'a TcpListener,
    cancelable: bool,
}

impl<'a> Accepting<'a> {
    pub(crate) fn new(listener: &'a TcpListener) -> Self {
        Self {
            listener,
            cancelable: true,
        }
    }

    pub fn cancelable(&mut self, v: bool) -> &mut Self {
        self.cancelable = v;
        self
    }
}

impl<'a> Future for Accepting<'a> {
    type Output = Result<(TcpStream, SocketAddr), std::io::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        // Accept the connection.
        match self.listener.accept() {
            Ok(r) => return Poll::Ready(Ok(r)),
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    return Poll::Ready(Err(e));
                }
            }
        }

        // Register for accept notification.
        let socket = get_underlying_socket(self.listener);

        register_pending(cx, socket, self.cancelable);
        get_dispatcher().watch_for_accept(socket);

        Poll::Pending
    }
}

/// Represents a [`Future`] for asynchronous [`TcpStream::read`].
pub struct Reading<'stream, 'buffer> {
    stream: &'stream mut TcpStream,
    buffer: &'buffer mut [u8],
    cancelable: bool,
}

impl<'stream, 'buffer> Reading<'stream, 'buffer> {
    pub(crate) fn new(stream: &'stream mut TcpStream, buffer: &'buffer mut [u8]) -> Self {
        Self {
            stream,
            buffer,
            cancelable: true,
        }
    }

    pub fn cancelable(&mut self, v: bool) -> &mut Self {
        self.cancelable = v;
        self
    }
}

impl<'stream, 'buffer> Future for Reading<'stream, 'buffer> {
    type Output = std::io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        // Read the data.
        let f = self.deref_mut();

        match f.stream.read(f.buffer) {
            Ok(r) => return Poll::Ready(Ok(r)),
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    return Poll::Ready(Err(e));
                }
            }
        }

        // Register for read notification.
        let socket = get_underlying_socket(f.stream);

        register_pending(cx, socket, self.cancelable);
        get_dispatcher().watch_for_read(socket);

        Poll::Pending
    }
}

/// Represents a [`Future`] for asynchronous [`TcpStream::write`].
pub struct Writing<'stream, 'data> {
    stream: &'stream mut TcpStream,
    data: &'data [u8],
    cancelable: bool,
}

impl<'stream, 'data> Writing<'stream, 'data> {
    pub(crate) fn new(stream: &'stream mut TcpStream, data: &'data [u8]) -> Self {
        Self {
            stream,
            data,
            cancelable: false,
        }
    }

    pub fn cancelable(&mut self, v: bool) -> &mut Self {
        self.cancelable = v;
        self
    }
}

impl<'stream, 'data> Future for Writing<'stream, 'data> {
    type Output = std::io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        // Send the data.
        let f = self.deref_mut();

        match f.stream.write(f.data) {
            Ok(r) => return Poll::Ready(Ok(r)),
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    return Poll::Ready(Err(e));
                }
            }
        }

        // Register for write notification.
        let socket = get_underlying_socket(f.stream);

        register_pending(cx, socket, self.cancelable);
        get_dispatcher().watch_for_write(socket);

        Poll::Pending
    }
}

#[cfg(target_family = "unix")]
fn get_underlying_socket<O: AsRawFd>(object: &O) -> std::os::unix::io::RawFd {
    object.as_raw_fd()
}

#[cfg(target_family = "windows")]
fn get_underlying_socket<O: AsRawSocket>(object: &O) -> std::os::windows::io::RawSocket {
    object.as_raw_socket()
}

fn register_pending(context: &Context, socket: Socket, cancelable: bool) {
    let data = PendingData {
        waker: context.waker().clone(),
        cancelable,
    };

    if unsafe { (*PENDING).insert(socket, data).is_some() } {
        panic!("This future cannot be polling while other futures still in-progress");
    }
}

fn get_dispatcher() -> &'static mut dyn Dispatcher {
    if let Some(d) = unsafe { DISPATCHER.clone() } {
        unsafe { &mut *d }
    } else {
        panic!("This future required Kami runtime");
    }
}
