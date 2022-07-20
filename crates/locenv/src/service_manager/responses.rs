use super::client::Response;
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
    fn code(&self) -> StatusCode {
        StatusCode::OK
    }
}
