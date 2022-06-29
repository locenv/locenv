use http::json::JsonReader;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Seek;

mod models;

#[derive(Debug)]
pub enum Error {
    InvalidIdentifier,
    ReadReleaseFailed(Box<dyn std::error::Error>),
    GetReleaseFailed(http::status::Code),
    DeserializeReleaseFailed(Box<dyn std::error::Error>),
    DownloadReleaseFailed(Box<dyn std::error::Error>),
}

pub fn get_latest_package(id: &str) -> Result<File, Error> {
    // Parse ID.
    let mut buffer = String::with_capacity(id.len());
    let mut owner: Option<String> = None;

    for c in id.chars() {
        if c == '/' {
            if owner.is_none() {
                if buffer.is_empty() {
                    return Err(Error::InvalidIdentifier);
                }

                owner = Some(buffer.clone());
                buffer.truncate(0);
            } else {
                return Err(Error::InvalidIdentifier);
            }
        } else {
            buffer.push(c);
        }
    }

    if owner.is_none() || buffer.is_empty() {
        return Err(Error::InvalidIdentifier);
    }

    let owner = owner.unwrap();
    let repo = buffer;

    // Get latest release.
    let mut handler = JsonReader::new();
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );

    let status = match http::get(&url, &mut handler) {
        Ok(r) => r,
        Err(e) => return Err(Error::ReadReleaseFailed(e.into())),
    };

    if status != http::status::OK {
        return Err(Error::GetReleaseFailed(status));
    }

    let release: models::Release = match handler.deserialize() {
        Ok(r) => r,
        Err(e) => return Err(Error::DeserializeReleaseFailed(e.into())),
    };

    // Download release asset.
    let mut asset = tempfile::tempfile().unwrap();
    let mut handler =
        http::writer::Writer::new(&asset).allow_type("application/zip".parse().unwrap());

    if let Err(e) = http::get(&release.assets[0].url, &mut handler) {
        return Err(Error::DownloadReleaseFailed(e.into()));
    }

    // Reset file position before return.
    asset.rewind().unwrap();

    Ok(asset)
}

// Error

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Error::InvalidIdentifier => write!(f, "Invalid repository identifier"),
            Error::ReadReleaseFailed(e) | Error::DeserializeReleaseFailed(e) => {
                write!(f, "Failed to read latest release: {}", e)
            }
            Error::GetReleaseFailed(c) => write!(f, "Failed to get latest release: {}", c),
            Error::DownloadReleaseFailed(e) => {
                write!(f, "Failed to download latest release: {}", e)
            }
        }
    }
}