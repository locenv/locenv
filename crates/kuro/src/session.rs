use crate::handler::Handler;
use crate::header::Header;

pub struct Session<'h, H: Handler> {
    handler: &'h mut H,
    invalid_header: Option<Vec<u8>>,
    status_error: Option<H::Err>,
    header_error: Option<H::Err>,
    body_error: Option<H::Err>,
}

impl<'h, H: Handler> Session<'h, H> {
    pub fn new(handler: &'h mut H) -> Self {
        Session {
            handler,
            invalid_header: None,
            status_error: None,
            header_error: None,
            body_error: None,
        }
    }

    pub fn invalid_header(&mut self) -> Option<Vec<u8>> {
        self.invalid_header.take()
    }

    pub fn status_error(&mut self) -> Option<H::Err> {
        self.status_error.take()
    }

    pub fn header_error(&mut self) -> Option<H::Err> {
        self.header_error.take()
    }

    pub fn body_error(&mut self) -> Option<H::Err> {
        self.body_error.take()
    }
}

impl<'h, H: Handler> curl::easy::Handler for Session<'h, H> {
    fn header(&mut self, data: &[u8]) -> bool {
        // Convert header line to Rust string.
        let line = match std::str::from_utf8(data) {
            Ok(r) => r,
            Err(_) => {
                self.invalid_header = Some(data.to_vec());
                return false;
            }
        };

        // Parse header.
        if let Some((name, value)) = line.split_once(':') {
            let value = value.trim();

            if name.is_empty() || value.is_empty() {
                self.invalid_header = Some(data.to_vec());
                return false;
            }

            let header: Header = match name.parse() {
                Ok(r) => r,
                Err(_) => {
                    self.invalid_header = Some(data.to_vec());
                    return false;
                }
            };

            if let Err(e) = self.handler.process_header(&header, value) {
                self.header_error = Some(e);
                return false;
            }
        } else if line == "\r\n" {
            if let Err(e) = self.handler.begin_body() {
                self.header_error = Some(e);
                return false;
            }
        } else if line.starts_with("HTTP/") {
            let status: crate::status::Line = match line.parse() {
                Ok(r) => r,
                Err(_) => {
                    self.invalid_header = Some(data.to_vec());
                    return false;
                }
            };

            if let Err(e) = self.handler.process_status(&status) {
                self.status_error = Some(e);
                return false;
            }
        } else {
            self.invalid_header = Some(data.to_vec());
            return false;
        }

        true
    }

    fn write(&mut self, data: &[u8]) -> Result<usize, curl::easy::WriteError> {
        let r = if let Err(e) = self.handler.process_body(data) {
            self.body_error = Some(e);
            usize::MAX // Don't use zero due to it is possible for d to have zero length.
        } else {
            data.len()
        };

        Ok(r)
    }
}
