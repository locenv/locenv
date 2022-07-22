use http::StatusCode;
use serde::Serialize;

#[derive(Serialize)]
pub struct ServiceManagerStatus {}

impl ServiceManagerStatus {
    pub fn new() -> Self {
        Self {}
    }
}

impl Response for ServiceManagerStatus {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

/// Represents a response to send back to the client.
pub trait Response: serde::Serialize {
    fn status_code(&self) -> StatusCode;
}
