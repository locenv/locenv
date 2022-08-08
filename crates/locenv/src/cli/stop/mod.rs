use super::{Command, ServiceManagerState};
use crate::service_manager::api::{Request, ServiceManagerStatus};
use crate::SUCCESS;
use clap::ArgMatches;
use context::Context;
use http::StatusCode;
use kuro::mime::{MediaType, APPLICATION_JSON};
use kuro::{Endpoint, Headers, StatusLine};
use kuro_macros::{kuro, FollowLocation, NoDefaultHeaders};
use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::Cursor;

pub(super) const COMMAND: Command = Command {
    name: "stop",
    specs: |name| clap::Command::new(name).about("Stop all running services"),
    run,
    service_manager_state: Some(ServiceManagerState::Running),
};

pub const PUT_STATUS_FAILED: u8 = 1;

fn run(context: &Context, _: &ArgMatches) -> u8 {
    // Send stop command.
    let port = context
        .project()
        .runtime(false)
        .unwrap()
        .service_manager(false)
        .unwrap()
        .port()
        .read()
        .unwrap();

    if let Err(e) = kuro::execute(PutStatus::new(port, ServiceManagerStatus::Stopping)) {
        eprintln!("Failed to stop Service Manager: {}", e);
        return PUT_STATUS_FAILED;
    }

    // TODO: Wait until Service Manager stopped.

    SUCCESS
}

#[derive(FollowLocation, NoDefaultHeaders)]
#[kuro(error = "PutStatusError")]
struct PutStatus {
    port: u16,
    request: Cursor<Vec<u8>>,
    response: Vec<u8>,
}

impl PutStatus {
    fn new(port: u16, status: ServiceManagerStatus) -> Self {
        Self {
            port,
            request: Cursor::new(serde_json::to_vec(&status).unwrap()),
            response: Vec::new(),
        }
    }
}

impl Endpoint for PutStatus {
    type Output = ();

    fn method<'a>(&'a self) -> &'a http::Method {
        Request::SetStatus.method()
    }

    fn url<'a>(&'a self) -> Cow<'a, str> {
        let port = self.port;
        let path = Request::SetStatus.path();

        format!("http://localhost:{}{}", port, path).into()
    }

    fn override_request_headers<'a>(&'a self, h: &mut Headers<'a>) {
        h.content_length = Some(self.request.get_ref().len() as _);
    }

    fn read_request_body(&mut self, output: &mut [u8]) -> Result<u64, Self::Err> {
        let mut output = Cursor::new(output);
        let result = std::io::copy(&mut self.request, &mut output).unwrap();

        Ok(result)
    }

    fn process_response_status(&mut self, line: &StatusLine) -> Result<(), Self::Err> {
        match line.code() {
            StatusCode::ACCEPTED | StatusCode::BAD_REQUEST => Ok(()),
            c => Err(PutStatusError::UnexpectedStatusCode(c)),
        }
    }

    fn begin_response_body(
        &mut self,
        t: Option<&MediaType>,
        _: Option<u64>,
    ) -> Result<(), Self::Err> {
        match t {
            Some(t) => {
                if t == &APPLICATION_JSON {
                    Ok(())
                } else {
                    Err(PutStatusError::InvalidContentType(t.to_owned()))
                }
            }
            None => Ok(()),
        }
    }

    fn process_response_body(&mut self, chunk: &[u8]) -> Result<(), Self::Err> {
        self.response.extend_from_slice(chunk);
        Ok(())
    }

    fn new_invalid_response_header(&self, line: &[u8]) -> Self::Err {
        PutStatusError::InvalidResponseHeader(line.into())
    }

    fn new_http_stack_error(&self, cause: curl::Error) -> Self::Err {
        PutStatusError::HttpStackFailed(cause)
    }

    fn build_output(self, status: StatusLine) -> Result<Self::Output, Self::Err> {
        match status.code() {
            StatusCode::ACCEPTED => Ok(()),
            StatusCode::BAD_REQUEST => Err(PutStatusError::BadRequest),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
enum PutStatusError {
    HttpStackFailed(curl::Error),
    InvalidResponseHeader(Vec<u8>),
    UnexpectedStatusCode(StatusCode),
    InvalidContentType(MediaType<'static>),
    BadRequest,
}

impl Error for PutStatusError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::HttpStackFailed(e) => Some(e),
            _ => None,
        }
    }
}

impl Display for PutStatusError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::HttpStackFailed(e) => write!(f, "{}", e),
            Self::InvalidResponseHeader(h) => write!(f, "header {:?} is not valid", h),
            Self::UnexpectedStatusCode(c) => write!(f, "unexpected status {}", c),
            Self::InvalidContentType(t) => write!(f, "unexpected content type {}", t),
            Self::BadRequest => f.write_str("request is not valid"),
        }
    }
}
