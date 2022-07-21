use http::{StatusCode, Version};
use std::str::FromStr;

/// Represetns a status line of the response.
pub struct Line {
    version: Version,
    code: StatusCode,
}

impl Line {
    pub fn version(&self) -> Version {
        self.version
    }

    pub fn code(&self) -> StatusCode {
        self.code
    }
}

impl FromStr for Line {
    type Err = LineError;

    fn from_str(s: &str) -> Result<Self, LineError> {
        let mut version: Option<Version> = None;
        let mut code: Option<StatusCode> = None;

        for p in s.splitn(3, ' ') {
            if version.is_none() {
                version = Some(match p {
                    "HTTP/1.1" => Version::HTTP_11,
                    "HTTP/2" => Version::HTTP_2,
                    _ => return Err(LineError::InvalidVersion(p.into())),
                });
            } else if code.is_none() {
                code = Some(p.parse().map_err(|_| LineError::InvalidCode)?);
            }
        }

        if let Some(v) = version {
            if let Some(c) = code {
                return Ok(Line {
                    version: v,
                    code: c,
                });
            }
        }

        Err(LineError::Malformed)
    }
}

pub enum LineError {
    Malformed,
    InvalidVersion(String),
    InvalidCode,
}
