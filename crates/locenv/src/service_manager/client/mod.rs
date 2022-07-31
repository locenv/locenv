pub use self::header::{Headers, RequestLine};

use self::header::HeaderError;
use super::api::Response;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::mem::MaybeUninit;
use std::net::{SocketAddr, TcpStream};
use std::vec::Drain;

mod header;

/// Represents a client for the Service Manager.
pub struct Client {
    connection: TcpStream,
    address: SocketAddr,
    receive_buffer: Vec<u8>,
    headers: Headers,
}

impl Client {
    pub fn new(connection: TcpStream, address: SocketAddr) -> Self {
        Self {
            connection,
            address,
            receive_buffer: Vec::new(),
            headers: Headers::new(),
        }
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
    }

    pub async fn receive<'a>(&'a mut self) -> Result<Request<'a>, ReceiveError> {
        let mut buffer: [u8; 8192] = unsafe { MaybeUninit::uninit().assume_init() };

        'read_data: loop {
            // Read some data from connection.
            let count = match kami::read(&mut self.connection, &mut buffer).await {
                Ok(r) => r,
                Err(e) => return Err(ReceiveError::ReadFailed(e)),
            };

            if count == 0 {
                return Err(ReceiveError::EndOfFile);
            }

            self.receive_buffer.extend_from_slice(&buffer[..count]);

            // Read headers.
            if !self.headers.is_complete() {
                loop {
                    if !self.decode_header()? {
                        continue 'read_data;
                    } else if self.headers.is_complete() {
                        break;
                    }
                }
            }

            // Get body.
            let content_length = self.headers.content_length().unwrap_or(0);

            if self.receive_buffer.len() < content_length {
                continue;
            }

            let request = Request {
                headers: &mut self.headers,
                body: self.receive_buffer.drain(..content_length),
            };

            break Ok(request);
        }
    }

    pub async fn send<R: Response>(&mut self, response: R) {
        let mut data: Vec<u8> = Vec::new();
        let body = if response.has_body() {
            serde_json::to_vec(&response).unwrap()
        } else {
            Vec::new()
        };

        // Write headers.
        write!(data, "HTTP/1.1 {}\r\n", response.status_code()).unwrap();
        write!(data, "Content-Type: application/json\r\n").unwrap();
        write!(data, "Content-Length: {}\r\n", body.len()).unwrap();
        write!(data, "\r\n").unwrap();

        // Write body.
        data.extend(body);

        // Write data.
        let mut total: usize = 0;

        while total < data.len() {
            let written = match kami::write(&mut self.connection, &data[total..]).await {
                Ok(r) => r,
                Err(e) => match e.kind() {
                    std::io::ErrorKind::Interrupted => continue,
                    _ => {
                        eprintln!(
                            "Failed to response {} to {}: {}",
                            response.status_code(),
                            self.address,
                            e
                        );
                        return;
                    }
                },
            };

            if written == 0 {
                eprintln!(
                    "Failed to response {} to {}: end of file has been reached",
                    response.status_code(),
                    self.address
                );
                return;
            }

            total += written;
        }
    }

    fn decode_header(&mut self) -> Result<bool, ReceiveError> {
        // Get header line.
        for i in 0..self.receive_buffer.len() {
            // Find '\r\n'.
            let remaining = &self.receive_buffer[i..self.receive_buffer.len()];

            if remaining.len() < 2 {
                return Ok(false);
            } else if remaining[0] != b'\r' {
                continue;
            } else if remaining[1] != b'\n' {
                return Err(ReceiveError::NotHttp);
            }

            // Extract header line.
            let line = match std::str::from_utf8(&self.receive_buffer[..i]) {
                Ok(r) => r,
                Err(_) => return Err(ReceiveError::NotHttp),
            };

            // Parse header.
            if let Err(e) = self.headers.parse(line) {
                return Err(match e {
                    HeaderError::Invalid => ReceiveError::NotHttp,
                    HeaderError::MethodNotSupported(m) => {
                        ReceiveError::MethodNotSupported(m.into())
                    }
                    HeaderError::TargetNotSupported(t) => {
                        ReceiveError::TargetNotSupported(t.into())
                    }
                });
            }

            // Remove processed data.
            self.receive_buffer.drain(0..(i + 2));

            return Ok(true);
        }

        Ok(false)
    }
}

/// Represents an HTTP request from client.
pub struct Request<'client> {
    headers: &'client mut Headers,
    body: Drain<'client, u8>,
}

impl<'client> Request<'client> {
    pub fn headers(&self) -> &Headers {
        self.headers
    }

    pub fn body(&self) -> &[u8] {
        self.body.as_slice()
    }
}

impl<'client> Drop for Request<'client> {
    fn drop(&mut self) {
        self.headers.clear();
    }
}

/// Represents an error when reading a request from the client.
#[derive(Debug)]
pub enum ReceiveError {
    ReadFailed(std::io::Error),
    EndOfFile,
    NotHttp,
    MethodNotSupported(String),
    TargetNotSupported(String),
}

impl Error for ReceiveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ReceiveError::ReadFailed(e) => Some(e),
            _ => None,
        }
    }
}

impl Display for ReceiveError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::ReadFailed(_) => f.write_str("read failed"),
            Self::EndOfFile => f.write_str("end of file has been reached"),
            Self::NotHttp => f.write_str("the request is not a valid HTTP"),
            Self::MethodNotSupported(name) => write!(f, "method '{}' is not supported", name),
            Self::TargetNotSupported(target) => write!(f, "target '{}' is not supported", target),
        }
    }
}
