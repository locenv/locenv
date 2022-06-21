use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone)]
pub struct Spec {
    reg: String,
    id: String,
}

#[derive(Debug)]
pub enum SpecParseError {
    InvalidFormat,
}

pub fn run(spec: &Spec) -> Result<(), Box<dyn Error>> {
    Ok(())
}

// Spec

impl FromStr for Spec {
    type Err = SpecParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (reg, id) = match s.split_once(':') {
            Some(v) => v,
            None => return Err(SpecParseError::InvalidFormat),
        };

        if reg.is_empty() || id.is_empty() {
            return Err(SpecParseError::InvalidFormat);
        }

        Ok(Spec {
            reg: reg.into(),
            id: id.into(),
        })
    }
}

// SpecParseError

impl Error for SpecParseError {}

impl Display for SpecParseError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::InvalidFormat => write!(f, "Unrecognized format"),
        }
    }
}
