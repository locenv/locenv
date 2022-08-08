use self::mime::MediaType;
use http::header::HeaderName;
use http::{Method, StatusCode, Version};
use std::borrow::Cow;
use std::cell::RefCell;
use std::ops::Deref;
use std::str::FromStr;

pub mod mime;

mod curl;

pub fn execute<E: Endpoint>(endpoint: E) -> Result<E::Output, E::Err> {
    let endpoint = RefCell::new(endpoint);

    // Setup client.
    let mut client = ::curl::easy::Easy2::new(self::curl::HandlerAdapter::new(&endpoint));

    setup_curl(&mut client, endpoint.borrow().deref());

    // Execute request.
    if let Err(e) = client.perform() {
        return Err(if let Some(v) = client.get_mut().take_error() {
            v
        } else {
            drop(client);
            endpoint.borrow().new_http_stack_error(e)
        });
    }

    drop(client);

    // Get output.
    endpoint.into_inner().build_output()
}

fn setup_curl<H, E>(client: &mut ::curl::easy::Easy2<H>, endpoint: &E)
where
    E: Endpoint,
{
    // Method and URL.
    match endpoint.method() {
        &Method::GET => client.get(true).unwrap(),
        &Method::PUT => client.put(true).unwrap(),
        m => panic!("Method {} is not implemented", m),
    }

    client.url(endpoint.url().as_ref()).unwrap();

    if endpoint.follow_location() {
        client.follow_location(true).unwrap();
    }

    // Custom headers.
    let mut headers = ::curl::easy::List::new();
    let mut custom = endpoint.default_request_headers();

    endpoint.override_request_headers(&mut custom);

    if let Some(v) = custom.content_length {
        client.in_filesize(v).unwrap();
    }

    if let Some(v) = custom.user_agent {
        client.useragent(v).unwrap();
    }

    if let Some(v) = custom.accept {
        headers.append(&format!("Accept: {}", v)).unwrap();
    }

    client.http_headers(headers).unwrap();
}

/// Represents how to send a request to a specified HTTP endpoint and how to handle the response.
pub trait Endpoint
where
    Self: FollowLocation + DefaultHeaders + Error,
{
    type Output;

    fn method<'a>(&'a self) -> &'a Method;
    fn url<'a>(&'a self) -> Cow<'a, str>;

    fn override_request_headers<'a>(&'a self, _: &mut Headers<'a>) {}

    fn read_request_body(&mut self, _: &mut [u8]) -> Result<usize, Self::Err> {
        Ok(0)
    }

    /// Check if status line is an expected one. This function will not called for redirection if
    /// [`FollowLocation::follow_location`] is `true`.
    fn process_response_status(&mut self, line: &StatusLine) -> Result<(), Self::Err>;

    fn process_response_header(&mut self, _: &HeaderName, _: &str) -> Result<(), Self::Err> {
        Ok(())
    }

    fn begin_response_body(
        &mut self,
        _: Option<&MediaType>,
        _: Option<u64>,
    ) -> Result<(), Self::Err> {
        Ok(())
    }

    fn process_response_body(&mut self, chunk: &[u8]) -> Result<(), Self::Err>;

    fn new_invalid_response_header(&self, line: &[u8]) -> Self::Err;
    fn new_http_stack_error(&self, cause: ::curl::Error) -> Self::Err;

    fn build_output(self) -> Result<Self::Output, Self::Err>;
}

pub trait FollowLocation {
    fn follow_location(&self) -> bool;
}

pub trait DefaultHeaders {
    fn default_request_headers<'a>(&self) -> Headers<'a>;
}

pub trait Error {
    type Err;
}

#[derive(Default)]
pub struct Headers<'endpoint> {
    pub content_length: Option<u64>,
    pub user_agent: Option<&'endpoint str>,
    pub accept: Option<&'endpoint str>,
}

/// Represetns a status line of the response.
pub struct StatusLine {
    version: Version,
    code: StatusCode,
}

impl StatusLine {
    pub fn version(&self) -> Version {
        self.version
    }

    pub fn code(&self) -> StatusCode {
        self.code
    }
}

impl FromStr for StatusLine {
    type Err = InvalidStatusLine;

    fn from_str(s: &str) -> Result<Self, InvalidStatusLine> {
        let mut version: Option<Version> = None;
        let mut code: Option<StatusCode> = None;

        for p in s.splitn(3, ' ') {
            if version.is_none() {
                version = Some(match p {
                    "HTTP/1.1" => Version::HTTP_11,
                    "HTTP/2" => Version::HTTP_2,
                    _ => return Err(InvalidStatusLine::InvalidVersion(p.into())),
                });
            } else if code.is_none() {
                code = Some(match p.parse() {
                    Ok(r) => r,
                    Err(_) => return Err(InvalidStatusLine::InvalidCode(p.into())),
                });
            }
        }

        if let (Some(v), Some(c)) = (version, code) {
            Ok(Self {
                version: v,
                code: c,
            })
        } else {
            Err(InvalidStatusLine::Malformed)
        }
    }
}

pub enum InvalidStatusLine {
    Malformed,
    InvalidVersion(String),
    InvalidCode(String),
}
