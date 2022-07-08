use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;

pub const APPLICATION_JSON: MediaType = MediaType {
    r#type: Cow::Borrowed("application"),
    subtype: Cow::Borrowed("json"),
};

// https://datatracker.ietf.org/doc/html/rfc2045#section-5
#[derive(Eq, Debug)]
pub struct MediaType<'t> {
    r#type: Cow<'t, str>,
    subtype: Cow<'t, str>,
}

impl<'t> Hash for MediaType<'t> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.r#type.to_lowercase().hash(state);
        self.subtype.to_lowercase().hash(state);
    }
}

impl<'t> PartialEq for MediaType<'t> {
    fn eq(&self, other: &Self) -> bool {
        self.r#type.to_lowercase() == other.r#type.to_lowercase()
            && self.subtype.to_lowercase() == other.subtype.to_lowercase()
    }
}

impl<'t> Display for MediaType<'t> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}/{}", self.r#type, self.subtype)
    }
}

impl<'t> FromStr for MediaType<'t> {
    type Err = MediaTypeError;

    fn from_str(s: &str) -> Result<Self, MediaTypeError> {
        // Type.
        let i = match s.find('/') {
            Some(v) => v,
            None => return Err(MediaTypeError::Malformed),
        };

        let r#type = &s[..i];

        if r#type.is_empty() {
            return Err(MediaTypeError::Malformed);
        }

        let remain = match s.get((i + 1)..) {
            Some(v) if !v.is_empty() => v,
            _ => return Err(MediaTypeError::Malformed),
        };

        // Subtype.
        let subtype = match remain.find(';') {
            Some(i) => {
                let t = &remain[..i];

                if t.is_empty() {
                    return Err(MediaTypeError::Malformed);
                }

                t
            }
            None => remain,
        };

        Ok(MediaType {
            r#type: Cow::Owned(r#type.into()),
            subtype: Cow::Owned(subtype.into()),
        })
    }
}

#[derive(Debug)]
pub enum MediaTypeError {
    Malformed,
    InvalidType,
    InvalidSubtype,
}
