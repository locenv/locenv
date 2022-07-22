use crate::service_manager::requests::Request;
use http::header::{HeaderName, CONTENT_LENGTH};
use http::{Method, Uri, Version};

pub struct State {
    request: Option<Request>,
    content_length: usize,
    is_body: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            request: None,
            content_length: 0,
            is_body: false,
        }
    }

    pub fn is_body(&self) -> bool {
        self.is_body
    }

    pub fn content_length(&self) -> usize {
        self.content_length
    }

    pub fn complete(&mut self) -> (Request, usize) {
        let request = self.request.take().unwrap();
        let content_length = self.content_length;

        self.content_length = 0;
        self.is_body = false;

        (request, content_length)
    }

    pub fn parse_header(&mut self, line: &str) -> Result<(), HeaderError> {
        if self.request.is_none() {
            // Parse the request line.
            let mut method: Option<Method> = None;
            let mut target: Option<Uri> = None;
            let mut version: Option<Version> = None;

            for component in line.split(' ') {
                if component.is_empty() {
                    return Err(HeaderError::Invalid);
                } else if method.is_none() {
                    method = Some(match component.parse() {
                        Ok(r) => match r {
                            Method::OPTIONS | Method::CONNECT => {
                                return Err(HeaderError::MethodNotSupported(component.into()))
                            }
                            m => m,
                        },
                        Err(_) => return Err(HeaderError::Invalid),
                    });
                } else if target.is_none() {
                    target = Some(match component.parse::<Uri>() {
                        Ok(r) => {
                            if r.scheme().is_some() || r.authority().is_some() {
                                return Err(HeaderError::TargetNotSupported(component.into()));
                            } else {
                                r
                            }
                        }
                        Err(_) => return Err(HeaderError::Invalid),
                    });
                } else if version.is_none() {
                    version = Some(match component {
                        "HTTP/1.1" => Version::HTTP_11,
                        _ => return Err(HeaderError::Invalid),
                    });
                } else {
                    return Err(HeaderError::Invalid);
                }
            }

            // Process the request line.
            if let Some(method) = method {
                if let Some(target) = target {
                    if version.is_some() {
                        if let Some(request) = Request::resolve(&method, target.path()) {
                            self.request = Some(request);
                            return Ok(());
                        }

                        return Err(HeaderError::NotFound(target.path().into()));
                    }
                }
            }

            return Err(HeaderError::Invalid);
        } else if line.is_empty() {
            self.is_body = true;
        } else if let Some(sep) = line.find(':') {
            let name = &line[..sep];
            let value = &line[(sep + 1)..];

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
                    self.content_length = match value.parse() {
                        Ok(r) => r,
                        Err(_) => return Err(HeaderError::Invalid),
                    }
                }
                _ => {}
            }
        } else {
            return Err(HeaderError::Invalid);
        }

        Ok(())
    }
}

pub enum HeaderError {
    Invalid,
    MethodNotSupported(String),
    TargetNotSupported(String),
    NotFound(String),
}
