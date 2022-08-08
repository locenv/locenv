use self::api::repos::{
    DownloadReleaseAsset, DownloadReleaseAssetError, GetLatestRelease, GetLatestReleaseError,
};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Seek;

mod api;

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
    let release = match kuro::execute(GetLatestRelease::new(&owner, &repo)) {
        Ok(r) => r,
        Err(e) => return Err(Error::GetReleaseFailed(e)),
    };

    // Download release asset.
    let mut asset = match kuro::execute(DownloadReleaseAsset::new(&release.assets[0].url)) {
        Ok(r) => r,
        Err(e) => return Err(Error::DownloadReleaseFailed(e)),
    };

    // Reset file position before return.
    asset.rewind().unwrap();

    Ok(asset)
}

#[derive(Debug)]
pub enum Error {
    InvalidIdentifier,
    GetReleaseFailed(GetLatestReleaseError),
    DownloadReleaseFailed(DownloadReleaseAssetError),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::GetReleaseFailed(e) => Some(e),
            Self::DownloadReleaseFailed(e) => Some(e),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::InvalidIdentifier => f.write_str("invalid package identifier"),
            Self::GetReleaseFailed(_) => f.write_str("get release failed"),
            Self::DownloadReleaseFailed(_) => f.write_str("download release failed"),
        }
    }
}
