use std::error::Error;
use std::fmt::{Display, Formatter, Result};
use std::path::PathBuf;

#[derive(Debug)]
pub struct CloneError {
    url: String,
    reason: git2::Error,
}

#[derive(Debug)]
pub struct OpenRepositoryError {
    path: PathBuf,
    reason: git2::Error,
}

#[derive(Debug)]
pub struct NotGitRepository {
    path: PathBuf,
}

#[derive(Debug)]
pub struct FindRemoteError {
    path: PathBuf,
    remote: String,
    reason: git2::Error,
}

#[derive(Debug)]
pub struct FetchError {
    path: PathBuf,
    remote: String,
    reason: git2::Error,
}

#[derive(Debug)]
pub struct MergeAnalysisError {
    path: PathBuf,
    reference: String,
    reason: git2::Error,
}

#[derive(Debug)]
pub struct FindReferenceError {
    path: PathBuf,
    name: String,
    reason: git2::Error,
}

#[derive(Debug)]
pub struct SetReferenceError {
    path: PathBuf,
    name: String,
    target: git2::Oid,
    reason: git2::Error,
}

#[derive(Debug)]
pub struct AnnotatedCommitFromReferenceError {
    path: PathBuf,
    reference: String,
    reason: git2::Error,
}

#[derive(Debug)]
pub struct SetHeadError {
    path: PathBuf,
    reference: String,
    reason: git2::Error,
}

#[derive(Debug)]
pub struct CheckoutError {
    path: PathBuf,
    reference: String,
    reason: git2::Error,
}

// GitCloneError

impl CloneError {
    pub fn new<U: Into<String>>(url: U, reason: git2::Error) -> Self {
        CloneError {
            url: url.into(),
            reason,
        }
    }
}

impl Error for CloneError {}

impl Display for CloneError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Failed to clone {}: {}", self.url, self.reason)
    }
}

// GitOpenError

impl OpenRepositoryError {
    pub fn new<P: Into<PathBuf>>(path: P, reason: git2::Error) -> Self {
        OpenRepositoryError {
            path: path.into(),
            reason,
        }
    }
}

impl Error for OpenRepositoryError {}

impl Display for OpenRepositoryError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "Failed to open {} as a Git repository: {}",
            self.path.display(),
            self.reason
        )
    }
}

// NotGitRepository

impl NotGitRepository {
    pub fn new(path: PathBuf) -> Self {
        NotGitRepository { path }
    }
}

impl Error for NotGitRepository {}

impl Display for NotGitRepository {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{} is not a Git repository", self.path.display())
    }
}

// GitFindRemoteError

impl FindRemoteError {
    pub fn new<P: Into<PathBuf>, R: Into<String>>(path: P, remote: R, reason: git2::Error) -> Self {
        FindRemoteError {
            path: path.into(),
            remote: remote.into(),
            reason,
        }
    }
}

impl Error for FindRemoteError {}

impl Display for FindRemoteError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "Failed to find '{}' remote of {}: {}",
            self.remote,
            self.path.display(),
            self.reason
        )
    }
}

// GitFetchError

impl FetchError {
    pub fn new<P: Into<PathBuf>, R: Into<String>>(path: P, remote: R, reason: git2::Error) -> Self {
        FetchError {
            path: path.into(),
            remote: remote.into(),
            reason,
        }
    }
}

impl Error for FetchError {}

impl Display for FetchError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "Failed to fetch '{}' for {}: {}",
            self.remote,
            self.path.display(),
            self.reason
        )
    }
}

// MergeAnalysisError

impl MergeAnalysisError {
    pub fn new<P: Into<PathBuf>, R: Into<String>>(
        path: P,
        reference: R,
        reason: git2::Error,
    ) -> Self {
        MergeAnalysisError {
            path: path.into(),
            reference: reference.into(),
            reason,
        }
    }
}

impl Error for MergeAnalysisError {}

impl Display for MergeAnalysisError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "Failed to analyze how to merge '{}' on {}: {}",
            self.reference,
            self.path.display(),
            self.reason
        )
    }
}

// FindReferenceError

impl FindReferenceError {
    pub fn new<P: Into<PathBuf>, N: Into<String>>(path: P, name: N, reason: git2::Error) -> Self {
        FindReferenceError {
            path: path.into(),
            name: name.into(),
            reason,
        }
    }
}

impl Error for FindReferenceError {}

impl Display for FindReferenceError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "Failed to find reference '{}' on {}: {}",
            self.name,
            self.path.display(),
            self.reason
        )
    }
}

// SetReferenceError

impl SetReferenceError {
    pub fn new<P: Into<PathBuf>, N: Into<String>>(
        path: P,
        name: N,
        target: git2::Oid,
        reason: git2::Error,
    ) -> Self {
        SetReferenceError {
            path: path.into(),
            name: name.into(),
            target,
            reason,
        }
    }
}

impl Error for SetReferenceError {}

impl Display for SetReferenceError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "Failed to set '{}' on {} to '{}': {}",
            self.name,
            self.path.display(),
            self.target,
            self.reason
        )
    }
}

// AnnotatedCommitFromReferenceError

impl AnnotatedCommitFromReferenceError {
    pub fn new<P: Into<PathBuf>, R: Into<String>>(
        path: P,
        reference: R,
        reason: git2::Error,
    ) -> Self {
        AnnotatedCommitFromReferenceError {
            path: path.into(),
            reference: reference.into(),
            reason,
        }
    }
}

impl Error for AnnotatedCommitFromReferenceError {}

impl Display for AnnotatedCommitFromReferenceError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "Failed to create annotated commit from {} for {}: {}",
            self.reference,
            self.path.display(),
            self.reason
        )
    }
}

// SetHeadError

impl SetHeadError {
    pub fn new<P: Into<PathBuf>, R: Into<String>>(
        path: P,
        reference: R,
        reason: git2::Error,
    ) -> Self {
        SetHeadError {
            path: path.into(),
            reference: reference.into(),
            reason,
        }
    }
}

impl Error for SetHeadError {}

impl Display for SetHeadError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "Failed to set HEAD of {} to '{}': {}",
            self.path.display(),
            self.reference,
            self.reason
        )
    }
}

// CheckoutError

impl CheckoutError {
    pub fn new<P: Into<PathBuf>, R: Into<String>>(
        path: P,
        reference: R,
        reason: git2::Error,
    ) -> Self {
        CheckoutError {
            path: path.into(),
            reference: reference.into(),
            reason,
        }
    }
}

impl Error for CheckoutError {}

impl Display for CheckoutError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "Failed to checkout {} on {}: {}",
            self.reference,
            self.path.display(),
            self.reason
        )
    }
}
