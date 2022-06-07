use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum FromFileError {
    OpenFailed(std::io::Error),
    ParseFailed(serde_yaml::Error),
}

// FromFileError

impl Error for FromFileError {}

impl Display for FromFileError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::OpenFailed(_) => f.write_str("Cannot open the specified file"),
            Self::ParseFailed(_) => f.write_str("The specified file has incorrect syntax"),
        }
    }
}
