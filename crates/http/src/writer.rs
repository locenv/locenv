use crate::handler::Handler;
use crate::header::Header;
use crate::mime::MediaType;
use crate::status;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::io::Write;

pub struct Writer<'allowed_type, O: Write> {
    output: O,
    allowed_types: HashSet<MediaType<'allowed_type>>,
    final_response: bool,
    content_type: Option<MediaType<'static>>,
}

#[derive(Debug)]
pub enum ReadError {
    UnhandledStatus(status::Code),
    MalformedContentType(String),
    UnexpectedContentType(Option<MediaType<'static>>),
    WriteFailed(std::io::Error),
}

// Writer

impl<'allowed_type, O: Write> Writer<'allowed_type, O> {
    pub fn new(output: O) -> Self {
        Writer {
            output,
            allowed_types: HashSet::new(),
            final_response: false,
            content_type: None,
        }
    }

    pub fn allow_type(mut self, r#type: MediaType<'allowed_type>) -> Self {
        self.allowed_types.insert(r#type);
        self
    }
}

impl<'allowed_type, O: Write> Handler for Writer<'allowed_type, O> {
    type Output = Option<MediaType<'static>>;
    type Err = ReadError;

    fn process_status(&mut self, line: &status::Line) -> Result<(), ReadError> {
        let code = line.code();

        if code == status::CONTINUE || code.is_redirection() {
            Ok(())
        } else if code.is_successful() {
            self.final_response = true;
            Ok(())
        } else {
            Err(ReadError::UnhandledStatus(code))
        }
    }

    fn process_header(&mut self, name: &Header, value: &str) -> Result<(), ReadError> {
        if !self.final_response {
            return Ok(());
        }

        match name {
            Header::ContentType => {
                let value: MediaType = match value.parse() {
                    Ok(r) => r,
                    Err(_) => return Err(ReadError::MalformedContentType(value.into())),
                };

                if !self.allowed_types.is_empty() && !self.allowed_types.contains(&value) {
                    return Err(ReadError::UnexpectedContentType(Some(value)));
                }

                self.content_type = Some(value);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn begin_body(&mut self) -> Result<(), ReadError> {
        if !self.final_response {
            return Ok(());
        } else if !self.allowed_types.is_empty() && self.content_type.is_none() {
            return Err(ReadError::UnexpectedContentType(None));
        }

        Ok(())
    }

    fn process_body(&mut self, chunk: &[u8]) -> Result<(), ReadError> {
        if !self.final_response {
            return Ok(());
        } else if let Err(e) = self.output.write_all(chunk) {
            return Err(ReadError::WriteFailed(e));
        }

        Ok(())
    }

    fn take_output(&mut self) -> Result<Option<MediaType<'static>>, ReadError> {
        Ok(self.content_type.take())
    }
}

// ReadError

impl std::error::Error for ReadError {}

impl Display for ReadError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ReadError::UnhandledStatus(c) => write!(f, "Unexpected status {}", c),
            ReadError::MalformedContentType(v) => write!(f, "Cannot parse content type {}", v),
            ReadError::UnexpectedContentType(t) => {
                if let Some(t) = t {
                    write!(f, "Unexpected content type {}", t)
                } else {
                    write!(f, "Unknow content type")
                }
            }
            ReadError::WriteFailed(e) => write!(f, "Failed to write output: {}", e),
        }
    }
}
