use http::StatusCode;
use reqmap_macros::HttpRequest;
use serde::{Deserialize, Serialize};

#[derive(HttpRequest)]
pub enum Request {
    #[put("/status")]
    SetStatus,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceManagerStatus {
    Running,
    Stopping,
}

/// Represents HTTP 202.
#[derive(Deserialize, Serialize)]
pub struct Accepted;

impl Response for Accepted {
    fn status_code(&self) -> StatusCode {
        StatusCode::ACCEPTED
    }

    fn has_body(&self) -> bool {
        false
    }
}

/// Represents HTTP 400.
#[derive(Deserialize, Serialize)]
pub struct BadRequest;

impl Response for BadRequest {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn has_body(&self) -> bool {
        false
    }
}

/// Represents HTTP 404.
#[derive(Deserialize, Serialize)]
pub struct NotFound;

impl Response for NotFound {
    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }

    fn has_body(&self) -> bool {
        false
    }
}

/// Represents a response to send back to the client.
pub trait Response: serde::Serialize {
    fn status_code(&self) -> StatusCode;
    fn has_body(&self) -> bool;
}
