use http::header::{HeaderName, CONTENT_LENGTH};
use http::{Method, Uri, Version};

pub struct Headers {
    request_line: Option<RequestLine>,
    content_length: Option<usize>,
    is_complete: bool,
}

impl Headers {
    pub(super) fn new() -> Self {
        Self {
            request_line: None,
            content_length: None,
            is_complete: false,
        }
    }

    pub fn request_line(&self) -> &RequestLine {
        self.request_line.as_ref().unwrap()
    }

    pub(super) fn content_length(&self) -> Option<usize> {
        self.content_length
    }

    pub(super) fn is_complete(&self) -> bool {
        self.is_complete
    }

    pub(super) fn clear(&mut self) {
        self.request_line = None;
        self.content_length = None;
        self.is_complete = false;
    }

    pub(super) fn parse<'a>(&mut self, line: &'a str) -> Result<(), HeaderError<'a>> {
        if self.request_line.is_none() {
            // Parse the line.
            let mut method: Option<Method> = None;
            let mut target: Option<Uri> = None;
            let mut version: Option<Version> = None;

            for c in line.split(' ') {
                if c.is_empty() {
                    return Err(HeaderError::Invalid);
                } else if method.is_none() {
                    method = Some(match c.parse() {
                        Ok(r) => match r {
                            Method::OPTIONS | Method::CONNECT => {
                                return Err(HeaderError::MethodNotSupported(c))
                            }
                            m => m,
                        },
                        Err(_) => return Err(HeaderError::Invalid),
                    });
                } else if target.is_none() {
                    target = Some(match c.parse::<Uri>() {
                        Ok(r) => {
                            if r.scheme().is_some() || r.authority().is_some() {
                                return Err(HeaderError::TargetNotSupported(c));
                            } else {
                                r
                            }
                        }
                        Err(_) => return Err(HeaderError::Invalid),
                    });
                } else if version.is_none() {
                    version = Some(match c {
                        "HTTP/1.1" => Version::HTTP_11,
                        _ => return Err(HeaderError::Invalid),
                    });
                } else {
                    return Err(HeaderError::Invalid);
                }
            }

            // Process the request line.
            if let (Some(m), Some(t), Some(v)) = (method, target, version) {
                self.request_line = Some(RequestLine::new(m, t, v));
            } else {
                return Err(HeaderError::Invalid);
            }
        } else if line.is_empty() {
            self.is_complete = true;
        } else if let Some(colon) = line.find(':') {
            let name = &line[..colon];
            let value = &line[(colon + 1)..];

            if name.is_empty() || value.is_empty() {
                return Err(HeaderError::Invalid);
            }

            // Parse name.
            let name: HeaderName = match name.parse() {
                Ok(r) => r,
                Err(_) => return Err(HeaderError::Invalid),
            };

            // Parse value.
            match &name {
                &CONTENT_LENGTH => {
                    self.content_length = Some(match value.parse() {
                        Ok(r) => r,
                        Err(_) => return Err(HeaderError::Invalid),
                    })
                }
                _ => {}
            }
        } else {
            return Err(HeaderError::Invalid);
        }

        Ok(())
    }
}

pub enum HeaderError<'line> {
    Invalid,
    MethodNotSupported(&'line str),
    TargetNotSupported(&'line str),
}

pub struct RequestLine {
    method: Method,
    target: Uri,
    version: Version,
}

impl RequestLine {
    fn new(method: Method, target: Uri, version: Version) -> Self {
        Self {
            method,
            target,
            version,
        }
    }

    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn target(&self) -> &Uri {
        &self.target
    }

    pub fn version(&self) -> Version {
        self.version
    }
}
