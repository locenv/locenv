use http::{Method, StatusCode};
use kuro::mime::{MediaType, APPLICATION_JSON};
use kuro::{Endpoint, Headers, StatusLine};
use kuro_macros::{kuro, FollowLocation};
use module_macros::GitHubHeaders;
use serde::Deserialize;
use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;

/// Gets information about the latest release.
///
/// See https://docs.github.com/en/rest/releases/releases#get-the-latest-release for more
/// information.
#[derive(GitHubHeaders, FollowLocation)]
#[kuro(error = "GetLatestReleaseError")]
pub struct GetLatestRelease<'owner, 'repo> {
    owner: &'owner str,
    repo: &'repo str,
    response: Vec<u8>,
}

impl<'owner, 'repo> GetLatestRelease<'owner, 'repo> {
    pub fn new(owner: &'owner str, repo: &'repo str) -> Self {
        Self {
            owner,
            repo,
            response: Vec::new(),
        }
    }
}

impl<'owner, 'repo> Endpoint for GetLatestRelease<'owner, 'repo> {
    type Output = Release;

    fn method<'a>(&'a self) -> &'a Method {
        &Method::GET
    }

    fn url<'a>(&'a self) -> Cow<'a, str> {
        let o = self.owner;
        let r = self.repo;

        format!("https://api.github.com/repos/{}/{}/releases/latest", o, r).into()
    }

    fn process_response_status(&mut self, line: &StatusLine) -> Result<(), Self::Err> {
        let c = line.code();

        if c == StatusCode::OK {
            Ok(())
        } else {
            Err(GetLatestReleaseError::UnexpectedStatusCode(c))
        }
    }

    fn begin_response_body(
        &mut self,
        ty: Option<&MediaType>,
        _: Option<u64>,
    ) -> Result<(), Self::Err> {
        if let Some(t) = ty {
            if t == &APPLICATION_JSON {
                Ok(())
            } else {
                Err(GetLatestReleaseError::InvalidContentType(t.to_owned()))
            }
        } else {
            Ok(())
        }
    }

    fn process_response_body(&mut self, chunk: &[u8]) -> Result<(), Self::Err> {
        self.response.extend_from_slice(chunk);
        Ok(())
    }

    fn new_invalid_response_header(&self, line: &[u8]) -> Self::Err {
        GetLatestReleaseError::InvalidResponseHeader(line.into())
    }

    fn new_http_stack_error(&self, cause: curl::Error) -> Self::Err {
        GetLatestReleaseError::HttpStackFailed(cause)
    }

    fn build_output(self) -> Result<Self::Output, Self::Err> {
        serde_json::from_slice(&self.response)
            .map_err(|_| GetLatestReleaseError::InvalidContent(self.response))
    }
}

#[derive(Deserialize)]
pub struct Release {
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Deserialize)]
pub struct ReleaseAsset {
    pub url: String,
}

#[derive(Debug)]
pub enum GetLatestReleaseError {
    HttpStackFailed(curl::Error),
    InvalidResponseHeader(Vec<u8>),
    UnexpectedStatusCode(StatusCode),
    InvalidContentType(MediaType<'static>),
    InvalidContent(Vec<u8>),
}

impl Error for GetLatestReleaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::HttpStackFailed(e) => Some(e),
            _ => None,
        }
    }
}

impl Display for GetLatestReleaseError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::HttpStackFailed(_) => f.write_str("HTTP stack failed"),
            Self::InvalidResponseHeader(h) => write!(f, "{:?} is not a valid header", h),
            Self::UnexpectedStatusCode(c) => write!(f, "unexpected status {}", c),
            Self::InvalidContentType(t) => write!(f, "unexpected content type {}", t),
            Self::InvalidContent(c) => write!(f, "unexpected content {:?}", c),
        }
    }
}

/// Download release asset.
///
/// See https://docs.github.com/en/rest/releases/assets#get-a-release-asset for more information.
#[derive(GitHubHeaders, FollowLocation)]
#[kuro(error = "DownloadReleaseAssetError")]
pub struct DownloadReleaseAsset<'url> {
    url: &'url str,
    file: File,
}

impl<'url> DownloadReleaseAsset<'url> {
    pub fn new(url: &'url str) -> Self {
        Self {
            url,
            file: tempfile::tempfile().unwrap(),
        }
    }
}

impl<'url> Endpoint for DownloadReleaseAsset<'url> {
    type Output = File;

    fn method<'a>(&'a self) -> &'a Method {
        &Method::GET
    }

    fn url<'a>(&'a self) -> Cow<'a, str> {
        self.url.into()
    }

    fn override_request_headers<'a>(&'a self, h: &mut Headers<'a>) {
        h.accept = Some("application/octet-stream");
    }

    fn process_response_status(&mut self, line: &StatusLine) -> Result<(), Self::Err> {
        let c = line.code();

        if c == StatusCode::OK {
            Ok(())
        } else {
            Err(DownloadReleaseAssetError::UnexpectedStatusCode(c))
        }
    }

    fn process_response_body(&mut self, chunk: &[u8]) -> Result<(), Self::Err> {
        self.file
            .write_all(chunk)
            .map_err(|e| DownloadReleaseAssetError::WriteFailed(e))
    }

    fn new_invalid_response_header(&self, line: &[u8]) -> Self::Err {
        DownloadReleaseAssetError::InvalidResponseHeader(line.into())
    }

    fn new_http_stack_error(&self, cause: curl::Error) -> Self::Err {
        DownloadReleaseAssetError::HttpStackFailed(cause)
    }

    fn build_output(self) -> Result<Self::Output, Self::Err> {
        Ok(self.file)
    }
}

#[derive(Debug)]
pub enum DownloadReleaseAssetError {
    HttpStackFailed(curl::Error),
    InvalidResponseHeader(Vec<u8>),
    UnexpectedStatusCode(StatusCode),
    WriteFailed(std::io::Error),
}

impl Error for DownloadReleaseAssetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::HttpStackFailed(e) => Some(e),
            Self::WriteFailed(e) => Some(e),
            _ => None,
        }
    }
}

impl Display for DownloadReleaseAssetError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::HttpStackFailed(_) => f.write_str("HTTP stack failed"),
            Self::InvalidResponseHeader(h) => write!(f, "{:?} is not a valid header", h),
            Self::UnexpectedStatusCode(c) => write!(f, "unexpected status {}", c),
            Self::WriteFailed(_) => f.write_str("write failed"),
        }
    }
}
