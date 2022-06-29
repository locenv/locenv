use std::borrow::Cow;
use std::str::FromStr;

pub enum Header<'c> {
    ContentType,
    Custom(Cow<'c, str>),
}

pub enum HeaderError {
    Invalid,
}

// Header

impl<'c> FromStr for Header<'c> {
    type Err = HeaderError;

    fn from_str(s: &str) -> Result<Self, HeaderError> {
        let v: Header<'c> = match s.to_lowercase().as_str() {
            "content-type" => Header::ContentType,
            v => {
                if v.starts_with("x-") {
                    Header::Custom(Cow::Owned(s.into()))
                } else {
                    return Err(HeaderError::Invalid);
                }
            }
        };

        Ok(v)
    }
}
