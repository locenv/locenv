use std::str::FromStr;

#[derive(Clone, Copy)]
pub enum Version {
    Http1_1,
    Http2,
}

pub enum VersionError {
    Invalid,
}

// Version

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, VersionError> {
        match s {
            "HTTP/1.1" => Ok(Version::Http1_1),
            "HTTP/2" => Ok(Version::Http2),
            _ => Err(VersionError::Invalid),
        }
    }
}
