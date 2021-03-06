use crate::handler::Handler;
use crate::header::Header;
use crate::mime::{MediaType, APPLICATION_JSON};
use crate::status;
use http::StatusCode;
use std::fmt::{Display, Formatter};

pub struct JsonReader {
    status: Option<StatusCode>,
    body: Vec<u8>,
}

#[derive(Debug)]
pub enum ReadError {
    UnhandledStatus(StatusCode),
    MalformedContentType(String),
    InvalidContentType(MediaType<'static>),
}

#[derive(Debug)]
pub enum DeserializeError {
    NoContent,
    InvalidBody(serde_json::Error),
}

// JsonReader

impl JsonReader {
    pub fn new() -> Self {
        JsonReader {
            status: None,
            body: Vec::new(),
        }
    }

    pub fn deserialize<'h, T: serde::de::Deserialize<'h>>(&'h self) -> Result<T, DeserializeError> {
        if self.body.is_empty() {
            return Err(DeserializeError::NoContent);
        }

        let json: T = match serde_json::from_slice(&self.body) {
            Ok(r) => r,
            Err(e) => return Err(DeserializeError::InvalidBody(e)),
        };

        Ok(json)
    }
}

impl Handler for JsonReader {
    type Output = StatusCode;
    type Err = ReadError;

    fn process_status(&mut self, line: &status::Line) -> Result<(), ReadError> {
        match line.code() {
            StatusCode::CONTINUE | StatusCode::MOVED_PERMANENTLY | StatusCode::FOUND => Ok(()),
            c @ (StatusCode::OK
            | StatusCode::CREATED
            | StatusCode::ACCEPTED
            | StatusCode::NON_AUTHORITATIVE_INFORMATION
            | StatusCode::NO_CONTENT
            | StatusCode::RESET_CONTENT
            | StatusCode::BAD_REQUEST
            | StatusCode::CONFLICT
            | StatusCode::GONE
            | StatusCode::PAYLOAD_TOO_LARGE
            | StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS) => {
                self.status = Some(c);
                Ok(())
            }
            c => Err(ReadError::UnhandledStatus(c)),
        }
    }

    fn process_header(&mut self, name: &Header, value: &str) -> Result<(), ReadError> {
        if self.status.is_none() {
            return Ok(());
        }

        match name {
            Header::ContentType => {
                let value: MediaType = match value.parse() {
                    Ok(r) => r,
                    Err(_) => return Err(ReadError::MalformedContentType(value.into())),
                };

                if value == APPLICATION_JSON {
                    Ok(())
                } else {
                    Err(ReadError::InvalidContentType(value))
                }
            }
            _ => Ok(()),
        }
    }

    fn begin_body(&mut self) -> Result<(), ReadError> {
        Ok(())
    }

    fn process_body(&mut self, chunk: &[u8]) -> Result<(), ReadError> {
        if let Some(c) = self.status {
            if c != StatusCode::NO_CONTENT {
                self.body.extend_from_slice(chunk);
            }
        }

        Ok(())
    }

    fn take_output(&mut self) -> Result<StatusCode, ReadError> {
        Ok(self.status.unwrap())
    }
}

// ReadError

impl std::error::Error for ReadError {}

impl Display for ReadError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ReadError::UnhandledStatus(c) => write!(f, "Unexpected status {}", c),
            ReadError::MalformedContentType(v) => write!(f, "Failed to parse content type {}", v),
            ReadError::InvalidContentType(t) => write!(f, "Unexpected content type {}", t),
        }
    }
}

// DeserializeError

impl std::error::Error for DeserializeError {}

impl Display for DeserializeError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            DeserializeError::NoContent => write!(f, "No content to deserialize"),
            DeserializeError::InvalidBody(e) => write!(f, "{}", e),
        }
    }
}
