use serde::de::{Deserialize, Deserializer, Error, Visitor};

#[derive(Debug, PartialEq)]
pub struct Url {
    pub user: Option<String>,
    pub host: String,
    pub path: Option<String>
}

impl Url {
    pub fn into_string(&self) -> String {
        let mut r = String::new();

        if let Some(v) = &self.user {
            r.push_str(v);
            r.push('@');
        }

        r.push_str(&self.host);

        if let Some(v) = &self.path {
            r.push(':');
            r.push_str(v);
        }

        r
    }
}

impl<'de> Deserialize<'de> for Url {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(UrlVisitor)
    }
}

struct UrlVisitor;

impl<'de> Visitor<'de> for UrlVisitor {
    type Value = Url;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a string in SCP-syntax")
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Url, E> {
        // Extract components.
        let mut buffer = String::with_capacity(v.len());
        let mut user: Option<String> = None;
        let mut host: Option<String> = None;

        for c in v.chars() {
            if host.is_none() {
                if c == '@' {
                    if user.is_some() || buffer.is_empty() {
                        return Err(E::custom("SCP-syntax is not valid"));
                    }

                    user = Some(buffer.clone());
                    buffer.truncate(0);
                } else if c == ':' {
                    if buffer.is_empty() {
                        return Err(E::custom("SCP-syntax is not valid"));
                    }

                    host = Some(buffer.clone());
                    buffer.truncate(0);
                } else {
                    buffer.push(c);
                }
            } else if c == '/' && buffer.is_empty() {
                return Err(E::custom("SCP-syntax is not valid"));
            } else {
                buffer.push(c);
            }
        }

        // Construct URL.
        if let Some(h) = host {
            let p = if buffer.is_empty() {
                None
            } else {
                buffer.shrink_to_fit();
                Some(buffer)
            };

            Ok(Url{ user, host: h, path: p })
        } else if user.is_none() && !buffer.is_empty() {
            buffer.shrink_to_fit();

            Ok(Url{ user: None, host: buffer, path: None })
        } else {
            Err(E::custom("SCP-syntax is not valid"))
        }
    }
}
