use std::io::{stdin, stdout, Read, Stdin, Stdout, Write};
use std::mem::MaybeUninit;

/// Represents a client for the Service Manager.
pub struct Client<C: Read + Write> {
    connection: C,
    buffer: Vec<u8>,
    body: bool,
    path: String,
    content_length: usize,
}

impl<C: Read + Write> Client<C> {
    pub(super) fn new(connection: C) -> Self {
        Self {
            connection,
            buffer: Vec::new(),
            body: false,
            path: String::new(),
            content_length: 0,
        }
    }

    pub fn receive(&mut self) -> Result<Request, ReceiveError> {
        let mut buffer: [u8; 8192] = unsafe { MaybeUninit::uninit().assume_init() };

        loop {
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
            if self.body {
                if self.buffer.len() < self.content_length {
                    continue;
                }

                return self.decode_body();
            } else {
                self.decode_headers()?;
            }
        }
    }

    fn decode_headers(&mut self) -> Result<(), ReceiveError> {
        Ok(())
    }

    fn decode_body(&mut self) -> Result<Request, ReceiveError> {
        Err(ReceiveError::UnknowRequest)
    }
}

/// Represents a request from the client.
pub enum Request {}

/// Represents an error when reading a request from the client.
#[derive(Debug)]
pub enum ReceiveError {
    ReadFailed(std::io::Error),
    EndOfFile,
    UnknowRequest,
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
