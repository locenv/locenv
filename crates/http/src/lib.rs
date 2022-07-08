use self::handler::Handler;
use self::session::Session;
use std::fmt::{Debug, Display, Formatter};

pub mod handler;
pub mod header;
pub mod json;
pub mod mime;
pub mod status;
pub mod version;
pub mod writer;

mod session;

pub struct Options<'user_agent, 'accept> {
    pub user_agent: Option<&'user_agent str>,
    pub accept: Option<&'accept str>,
}

#[derive(Debug)]
pub enum Error<H: Debug + Display> {
    RequestFailed(Box<dyn std::error::Error>),
    InvalidHeader(Vec<u8>),
    StatusFailed(H),
    HeaderFailed(H),
    BodyFailed(H),
    OutputFailed(H),
}

pub fn get<H: Handler>(
    url: &str,
    options: Option<&Options>,
    handler: &mut H,
) -> Result<H::Output, Error<H::Err>> {
    // Setup client.
    let mut client = curl::easy::Easy2::new(Session::new(handler));

    client.get(true).unwrap();
    client.url(url).unwrap();
    client.follow_location(true).unwrap();

    if let Some(o) = options {
        let mut customs = curl::easy::List::new();

        if let Some(v) = o.user_agent {
            client.useragent(v).unwrap();
        }

        if let Some(v) = o.accept {
            customs.append(&format!("Accept: {}", v)).unwrap();
        }

        client.http_headers(customs).unwrap();
    }

    // Execute request.
    if let Err(e) = client.perform() {
        let session = client.get_mut();
        let error = if let Some(e) = session.invalid_header() {
            Error::InvalidHeader(e)
        } else if let Some(e) = session.status_error() {
            Error::StatusFailed(e)
        } else if let Some(e) = session.header_error() {
            Error::HeaderFailed(e)
        } else if let Some(e) = session.body_error() {
            Error::BodyFailed(e)
        } else {
            Error::RequestFailed(e.into())
        };

        return Err(error);
    }

    drop(client);

    // Get output.
    let output = match handler.take_output() {
        Ok(r) => r,
        Err(e) => return Err(Error::OutputFailed(e)),
    };

    Ok(output)
}

// Error

impl<H: Debug + Display> std::error::Error for Error<H> {}

impl<H: Debug + Display> Display for Error<H> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Error::RequestFailed(e) => write!(f, "{}", e),
            Error::InvalidHeader(d) => {
                if let Ok(v) = std::str::from_utf8(d) {
                    write!(f, "Header {} is not valid", v)
                } else {
                    write!(f, "Header {:02x?} is not valid", d)
                }
            }
            Error::StatusFailed(e) => write!(f, "{}", e),
            Error::HeaderFailed(e) => write!(f, "{}", e),
            Error::BodyFailed(e) => write!(f, "{}", e),
            Error::OutputFailed(e) => write!(f, "{}", e),
        }
    }
}
