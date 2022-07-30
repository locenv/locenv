use http::StatusCode;
use reqmap_macros::HttpRequest;
use serde::{Deserialize, Serialize};

#[derive(HttpRequest)]
pub enum Request {
    #[put("/status")]
    SetStatus,
}

#[derive(Deserialize, Serialize)]
pub struct NotFound {}

impl Response for NotFound {
    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }
}

/// Represents a response to send back to the client.
pub trait Response: serde::Serialize {
    fn status_code(&self) -> StatusCode;
}
