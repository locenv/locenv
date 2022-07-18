use std::io::{stdin, stdout, Read, Stdin, Stdout, Write};

/// Represents a client for the Service Manager.
pub(super) struct Client<C: Read + Write> {
    connection: C,
}

impl<C: Read + Write> Client<C> {
    pub fn new(connection: C) -> Self {
        Self { connection }
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
