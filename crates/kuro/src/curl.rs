use crate::mime::MediaType;
use crate::{Endpoint, StatusLine};
use curl::easy::{Handler, ReadError, WriteError};
use http::header::{HeaderName, CONTENT_LENGTH, CONTENT_TYPE};
use std::cell::RefCell;

pub struct HandlerAdapter<'e, E: Endpoint> {
    endpoint: &'e RefCell<E>,
    status: Option<StatusLine>,
    content_type: Option<MediaType<'static>>,
    content_length: Option<u64>,
    state: State,
    error: Option<E::Err>,
}

impl<'e, E: Endpoint> HandlerAdapter<'e, E> {
    pub fn new(endpoint: &'e RefCell<E>) -> Self {
        Self {
            endpoint,
            status: None,
            content_type: None,
            content_length: None,
            state: State::Created,
            error: None,
        }
    }

    pub fn take_error(&mut self) -> Option<E::Err> {
        self.error.take()
    }
}

impl<'e, E: Endpoint> Handler for HandlerAdapter<'e, E> {
    fn read(&mut self, data: &mut [u8]) -> Result<usize, ReadError> {
        let count = match self.endpoint.borrow_mut().read_request_body(data) {
            Ok(r) => r,
            Err(e) => {
                self.error = Some(e);
                return Err(ReadError::Abort);
            }
        };

        Ok(count)
    }

    fn header(&mut self, data: &[u8]) -> bool {
        let mut endpoint = self.endpoint.borrow_mut();

        // Convert header line to Rust string.
        let data = &data[..(data.len() - 2)]; // Remove trailing \r\n.
        let line = match std::str::from_utf8(data) {
            Ok(r) => r,
            Err(_) => {
                self.error = Some(endpoint.new_invalid_response_header(data));
                return false;
            }
        };

        // Parse header.
        if let (Some(status), State::ReadingHeaders) = (&self.status, &self.state) {
            if let Some((name, value)) = line.split_once(':') {
                // Do a simple check first.
                let value = value.trim();

                if name.is_empty() || value.is_empty() {
                    self.error = Some(endpoint.new_invalid_response_header(data));
                    return false;
                }

                // Parse name.
                let name: HeaderName = match name.parse() {
                    Ok(r) => r,
                    Err(_) => {
                        self.error = Some(endpoint.new_invalid_response_header(data));
                        return false;
                    }
                };

                // Check redirection.
                if endpoint.follow_location() && status.code().is_redirection() {
                    return true;
                }

                // Capture value.
                match &name {
                    &CONTENT_TYPE => match value.parse::<MediaType>() {
                        Ok(r) => self.content_type = Some(r),
                        Err(_) => {
                            self.error = Some(endpoint.new_invalid_response_header(data));
                            return false;
                        }
                    },
                    &CONTENT_LENGTH => match value.parse::<u64>() {
                        Ok(r) => self.content_length = Some(r),
                        Err(_) => {
                            self.error = Some(endpoint.new_invalid_response_header(data));
                            return false;
                        }
                    },
                    _ => {}
                }

                // Invoke handler.
                if let Err(e) = endpoint.process_response_header(&name, value) {
                    self.error = Some(e);
                    return false;
                }
            } else if line.is_empty() {
                if !endpoint.follow_location() || !status.code().is_redirection() {
                    let ty = self.content_type.as_ref();
                    let len = self.content_length;

                    if let Err(e) = endpoint.begin_response_body(ty, len) {
                        self.error = Some(e);
                        return false;
                    }
                }

                self.state = State::ReadingBody;
            } else {
                self.error = Some(endpoint.new_invalid_response_header(data));
                return false;
            }
        } else {
            let status: StatusLine = match line.parse() {
                Ok(r) => r,
                Err(_) => {
                    self.error = Some(endpoint.new_invalid_response_header(data));
                    return false;
                }
            };

            if !endpoint.follow_location() || !status.code().is_redirection() {
                if let Err(e) = endpoint.process_response_status(&status) {
                    self.error = Some(e);
                    return false;
                }
            }

            self.status = Some(status);
            self.state = State::ReadingHeaders;
        }

        true
    }

    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        let mut endpoint = self.endpoint.borrow_mut();

        // Check redirection.
        let status = self.status.as_ref().unwrap();

        if endpoint.follow_location() && status.code().is_redirection() {
            return Ok(data.len());
        }

        // Process chunk.
        let result = if let Err(e) = endpoint.process_response_body(data) {
            self.error = Some(e);
            usize::MAX // Don't use zero due to it is possible for d to have zero length.
        } else {
            data.len()
        };

        Ok(result)
    }
}

enum State {
    Created,
    ReadingHeaders,
    ReadingBody,
}
