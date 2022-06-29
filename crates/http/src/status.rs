use crate::version::Version;
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::num::IntErrorKind;
use std::str::FromStr;

pub const CONTINUE: Code = Code(100);
pub const OK: Code = Code(200);
pub const CREATED: Code = Code(201);
pub const ACCEPTED: Code = Code(202);
pub const NON_AUTHORITATIVE_INFORMATION: Code = Code(203);
pub const NO_CONTENT: Code = Code(204);
pub const RESET_CONTENT: Code = Code(205);
pub const MOVED_PERMANENTLY: Code = Code(301);
pub const FOUND: Code = Code(302);
pub const BAD_REQUEST: Code = Code(400);
pub const CONFLICT: Code = Code(409);
pub const GONE: Code = Code(410);
pub const PAYLOAD_TOO_LARGE: Code = Code(413);
pub const UNAVAILABLE_FOR_LEGAL_REASONS: Code = Code(451);

pub struct Line<'r> {
    version: Version,
    code: Code,
    reason: Option<Cow<'r, str>>,
}

pub enum LineError {
    Malformed,
    InvalidVersion(String),
    InvalidCode(CodeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Code(u16);

pub enum CodeError {
    NaN,
    Invalid,
}

// Line

impl<'r> Line<'r> {
    pub fn version(&self) -> Version {
        self.version
    }

    pub fn code(&self) -> Code {
        self.code
    }

    pub fn reason(&self) -> Option<&Cow<'r, str>> {
        self.reason.as_ref()
    }
}

impl<'r> FromStr for Line<'r> {
    type Err = LineError;

    fn from_str(s: &str) -> Result<Self, LineError> {
        let mut version: Option<Version> = None;
        let mut code: Option<Code> = None;
        let mut reason: Option<&str> = None;

        for p in s.splitn(2, ' ') {
            if version.is_none() {
                version = Some(match p.parse() {
                    Ok(r) => r,
                    Err(_) => return Err(LineError::InvalidVersion(p.into())),
                });
            } else if code.is_none() {
                code = Some(match p.parse() {
                    Ok(r) => r,
                    Err(e) => return Err(LineError::InvalidCode(e)),
                });
            } else {
                reason = Some(p);
            }
        }

        if version.is_none() || code.is_none() {
            return Err(LineError::Malformed);
        }

        Ok(Line {
            version: version.unwrap(),
            code: code.unwrap(),
            reason: if let Some(v) = reason {
                Some(Cow::Owned(v.into()))
            } else {
                None
            },
        })
    }
}

// Code

impl Code {
    pub fn is_successful(self) -> bool {
        self.0 >= 200 && self.0 <= 299
    }

    pub fn is_redirection(self) -> bool {
        self.0 >= 300 && self.0 <= 399
    }
}

impl FromStr for Code {
    type Err = CodeError;

    fn from_str(s: &str) -> Result<Self, CodeError> {
        let code: u16 = match s.parse() {
            Ok(r) => r,
            Err(e) => {
                return Err(if *e.kind() == IntErrorKind::PosOverflow {
                    CodeError::Invalid
                } else {
                    CodeError::NaN
                })
            }
        };

        if code < 100 || code > 599 {
            return Err(CodeError::Invalid);
        }

        Ok(Code(code))
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
