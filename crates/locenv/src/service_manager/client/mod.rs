use self::request::{HeaderError, State};
use http::StatusCode;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::{stdin, stdout, Read, Stdin, Stdout, Write};
use std::mem::MaybeUninit;

mod request;

/// Represents a client for the Service Manager.
pub struct Client<C: Read + Write> {
    connection: C,
    buffer: Vec<u8>,
    state: State,
}

impl<C: Read + Write> Client<C> {
    pub(super) fn new(connection: C) -> Self {
        Self {
            connection,
            buffer: Vec::new(),
            state: State::new(),
        }
    }

    pub fn receive(&mut self) -> Result<Request, ReceiveError> {
        let mut buffer: [u8; 8192] = unsafe { MaybeUninit::uninit().assume_init() };

        'read_data: loop {
            // Read some data from connection.
            let count = self
                .connection
                .read(&mut buffer)
                .map_err(|e| ReceiveError::ReadFailed(e))?;

            if count == 0 {
                return Err(ReceiveError::EndOfFile);
            }

            self.buffer.extend_from_slice(&buffer[..count]);

            // Process buffer.
            if !self.state.is_body() {
                loop {
                    if !self.decode_header()? {
                        continue 'read_data;
                    } else if self.state.is_body() {
                        break;
                    }
                }
            }

            if self.buffer.len() < self.state.content_length() {
                continue;
            }

            return self.decode_body();
        }
    }

    pub fn send<R: Response>(&mut self, response: R) -> Result<(), SendError> {
        let body = serde_json::to_vec(&response).unwrap();
        let mut data: Vec<u8> = Vec::new();

        // Write headers.
        write!(data, "HTTP/1.1 {}\r\n", response.code()).unwrap();
        write!(data, "Content-Type: application/json\r\n").unwrap();
        write!(data, "Content-Length: {}\r\n", body.len()).unwrap();
        write!(data, "\r\n").unwrap();

        // Write body.
        data.extend(body);

        // Write data.
        let mut total: usize = 0;

        while total < data.len() {
            let written = match self.connection.write(&data[total..]) {
                Ok(r) => r,
                Err(e) => match e.kind() {
                    std::io::ErrorKind::Interrupted | std::io::ErrorKind::WouldBlock => continue,
                    _ => return Err(SendError::WriteFailed(e)),
                },
            };

            if written == 0 {
                return Err(SendError::EndOfFile);
            }

            total += written;
        }

        Ok(())
    }

    fn decode_header(&mut self) -> Result<bool, ReceiveError> {
        // Get header line.
        for i in 0..self.buffer.len() {
            // Find '\r\n'.
            let remaining = &self.buffer[i..self.buffer.len()];

            if remaining.len() < 2 {
                return Ok(false);
            } else if remaining[0] != b'\r' {
                continue;
            } else if remaining[1] != b'\n' {
                return Err(ReceiveError::NotHttp);
            }

            // Extract header line.
            let line = match std::str::from_utf8(&self.buffer[0..i]) {
                Ok(r) => r,
                Err(_) => return Err(ReceiveError::NotHttp),
            };

            // Parse header.
            if let Err(e) = self.state.parse_header(line) {
                return Err(match e {
                    HeaderError::Invalid => ReceiveError::NotHttp,
                    HeaderError::MethodNotSupported(method) => {
                        ReceiveError::MethodNotSupported(method)
                    }
                    HeaderError::TargetNotSupported(target) => {
                        ReceiveError::TargetNotSupported(target)
                    }
                    HeaderError::NotFound(_) => ReceiveError::UnknowRequest,
                });
            }

            // Remove processed data.
            self.buffer.drain(0..(i + 2));

            return Ok(true);
        }

        Ok(false)
    }

    fn decode_body(&mut self) -> Result<Request, ReceiveError> {
        // Extract data.
        let (request, content_length) = self.state.complete();
        let body = self.buffer.drain(..content_length);
        let data = body.as_slice();

        match request {
            request::Request::GetStatus => {
                if data.is_empty() {
                    Ok(Request::GetStatus)
                } else {
                    Err(ReceiveError::InvalidRequest)
                }
            }
        }
    }
}

/// Represents a request from the client.
#[derive(Debug)]
pub enum Request {
    GetStatus,
}

/// Represents an error when reading a request from the client.
#[derive(Debug)]
pub enum ReceiveError {
    ReadFailed(std::io::Error),
    EndOfFile,
    NotHttp,
    MethodNotSupported(String),
    TargetNotSupported(String),
    UnknowRequest,
    InvalidRequest,
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
            Self::ReadFailed(_) => f.write_str("failed to read data"),
            Self::EndOfFile => f.write_str("end of file has been reached"),
            Self::NotHttp => f.write_str("the request is not a valid HTTP"),
            Self::MethodNotSupported(name) => write!(f, "method '{}' is not supported", name),
            Self::TargetNotSupported(target) => write!(f, "target '{}' is not supported", target),
            Self::UnknowRequest => f.write_str("unknow request"),
            Self::InvalidRequest => f.write_str("invalid request"),
        }
    }
}

/// Represents a response to send back to the client.
pub trait Response: serde::Serialize {
    fn code(&self) -> StatusCode;
}

/// Represents an error when sending response back to the client failed.
#[derive(Debug)]
pub enum SendError {
    WriteFailed(std::io::Error),
    EndOfFile,
}

impl Error for SendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::WriteFailed(e) => Some(e),
            _ => None,
        }
    }
}

impl Display for SendError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::WriteFailed(_) => f.write_str("failed to write data"),
            Self::EndOfFile => f.write_str("end of file has been reached"),
        }
    }
}

/// Represents a connection that connect to STDIN and STDOUT.
pub(super) struct ConsoleConnection {
    stdin: Stdin,
    stdout: Stdout,
}

impl ConsoleConnection {
    pub fn new() -> Self {
        Self {
            stdin: stdin(),
            stdout: stdout(),
        }
    }
}

impl Read for ConsoleConnection {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stdin.read(buf)
    }
}

impl Write for ConsoleConnection {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stdout.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stdout.flush()
    }
}
