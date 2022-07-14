use super::RepositoryConfigurations;
use super::RepositoryType;
use std::path::Path;

mod git;

pub fn download<D: AsRef<Path>>(
    config: &RepositoryConfigurations,
    destination: D,
) -> Result<(), DownloadError> {
    let mut guard = UpdateGuard::new(destination.as_ref());

    match &config.r#type {
        RepositoryType::Git => git::clone(&config.uri, destination.as_ref(), &config.options)?,
    }

    guard.success = true;
    Ok(())
}

pub fn update<P: AsRef<Path>>(
    config: &RepositoryConfigurations,
    path: P,
) -> Result<(), UpdateError> {
    let mut guard = UpdateGuard::new(path.as_ref());

    match &config.r#type {
        RepositoryType::Git => git::pull(path.as_ref(), &config.options)?,
    }

    guard.success = true;
    Ok(())
}

pub enum DownloadError {
    InvalidOption(&'static str),
    GitCloneFailed(git2::Error),
}

impl From<git::CloneError> for DownloadError {
    fn from(e: git::CloneError) -> Self {
        match e {
            git::CloneError::InvalidOption(key) => Self::InvalidOption(key),
            git::CloneError::CloneFailed(e) => Self::GitCloneFailed(e),
        }
    }
}

pub enum UpdateError {
    InvalidOption(&'static str),
    GitOpenFailed(git2::Error),
    GitFindOriginFailed(git2::Error),
    GitFetchOriginFailed(git2::Error),
}

impl From<git::PullError> for UpdateError {
    fn from(e: git::PullError) -> Self {
        match e {
            git::PullError::RepositoryOpenFailed(e) => Self::GitOpenFailed(e),
            git::PullError::FindOriginFailed(e) => Self::GitFindOriginFailed(e),
            git::PullError::FetchOriginFailed(e) => Self::GitFetchOriginFailed(e),
            git::PullError::InvalidOption(name) => Self::InvalidOption(name),
        }
    }
}

struct UpdateGuard<'destination> {
    destination: &'destination Path,
    success: bool,
}

impl<'destination> UpdateGuard<'destination> {
    fn new(destination: &'destination Path) -> Self {
        Self {
            destination,
            success: false,
        }
    }
}

impl<'destination> Drop for UpdateGuard<'destination> {
    fn drop(&mut self) {
        if !self.success {
            std::fs::remove_dir_all(self.destination).unwrap();
        }
    }
}
