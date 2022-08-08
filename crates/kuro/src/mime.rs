use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::str::FromStr;

pub const APPLICATION_JSON: MediaType = MediaType {
    t: Cow::Borrowed("application"),
    s: Cow::Borrowed("json"),
};

// https://datatracker.ietf.org/doc/html/rfc2045#section-5
#[derive(Debug, Eq)]
pub struct MediaType<'t> {
    t: Cow<'t, str>,
    s: Cow<'t, str>,
}

impl<'t> MediaType<'t> {
    pub fn to_owned(&self) -> MediaType<'static> {
        MediaType::<'static> {
            t: Cow::Owned(self.t.as_ref().into()),
            s: Cow::Owned(self.s.as_ref().into()),
        }
    }
}

impl<'t> Hash for MediaType<'t> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for c in self.t.chars().flat_map(|c| c.to_lowercase()) {
            c.hash(state);
        }

        for c in self.s.chars().flat_map(|c| c.to_lowercase()) {
            c.hash(state);
        }
    }
}

impl<'t> PartialEq for MediaType<'t> {
    fn eq(&self, other: &Self) -> bool {
        // Compare type.
        let mut r = other.t.chars();
        for l in self.t.chars() {
            match r.next() {
                Some(r) => {
                    if !l.to_lowercase().eq(r.to_lowercase()) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Compare subtype.
        let mut r = other.s.chars();
        for l in self.s.chars() {
            match r.next() {
                Some(r) => {
                    if !l.to_lowercase().eq(r.to_lowercase()) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        true
    }
}

impl<'t> Display for MediaType<'t> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}/{}", self.t, self.s)
    }
}

impl FromStr for MediaType<'static> {
    type Err = InvalidMediaType;

    fn from_str(s: &str) -> Result<Self, InvalidMediaType> {
        // Type.
        let i = match s.find('/') {
            Some(v) => v,
            None => return Err(InvalidMediaType::Malformed),
        };

        let t = &s[..i];

        if t.is_empty() {
            return Err(InvalidMediaType::Malformed);
        }

        let s = match s.get((i + 1)..) {
            Some(v) if !v.is_empty() => v,
            _ => return Err(InvalidMediaType::Malformed),
        };

        // Subtype.
        let s = match s.find(';') {
            Some(i) => {
                let t = &s[..i];

                if t.is_empty() {
                    return Err(InvalidMediaType::Malformed);
                }

                t
            }
            None => s,
        };

        Ok(MediaType {
            t: Cow::Owned(t.into()),
            s: Cow::Owned(s.into()),
        })
    }
}

#[derive(Debug)]
pub enum InvalidMediaType {
    Malformed,
    InvalidType,
    InvalidSubtype,
}
