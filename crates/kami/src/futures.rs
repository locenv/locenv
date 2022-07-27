use crate::state::{DISPATCHER, WAKERS};
use std::future::Future;
use std::net::{SocketAddr, TcpListener, TcpStream};
#[cfg(target_family = "unix")]
use std::os::unix::prelude::AsRawFd;
#[cfg(target_family = "windows")]
use std::os::windows::io::AsRawSocket;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Represents a [`Future`] for asynchronous [`TcpListener::accept()`].
pub struct Accepting<'a>(&'a TcpListener);

impl<'a> Accepting<'a> {
    pub fn new(listener: &'a TcpListener) -> Self {
        Self(listener)
    }
}

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

        if let Some(_) = unsafe { (*WAKERS).insert(socket, cx.waker().clone()) } {
            panic!("This future cannot be polling while other futures still in-progress");
        }

        if let Some(d) = unsafe { DISPATCHER.clone() } {
            unsafe { (*d).watch_for_accept(socket) };
        } else {
            panic!("This future required Kami runtime");
        }

        Poll::Pending
    }
}
