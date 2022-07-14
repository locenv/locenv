use crate::RepositoryUri;
use git2::build::RepoBuilder;
use git2::{Cred, FetchOptions, RemoteCallbacks, Repository};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;

pub fn clone<D: AsRef<Path>>(
    uri: &RepositoryUri,
    dest: D,
    options: &HashMap<String, serde_yaml::Value>,
) -> Result<(), CloneError> {
    let mut repo = RepoBuilder::new();

    // Specify branch to clone.
    match get_branch_to_clone(options) {
        Ok(branch) => {
            repo.branch(branch);
            repo.remote_create(move |repo, name, url| {
                // https://git-scm.com/book/en/v2/Git-Internals-The-Refspec
                let spec = format!("+refs/heads/{0:}:refs/remotes/origin/{0:}", branch);

                repo.remote_with_fetch(name, url, &spec)
            });
        }
        Err(e) => match e {
            GetBranchToCloneError::NoOption => {}
            GetBranchToCloneError::InvalidValue(key) => return Err(CloneError::InvalidOption(key)),
        },
    };

    // Get remote URL.
    let url: Cow<str> = match uri {
        RepositoryUri::Scp(url) => {
            let mut options = FetchOptions::new();
            let mut callbacks = RemoteCallbacks::new();

            // Setup credential callback.
            callbacks.credentials(|_, username, _| {
                let mut private = dirs::home_dir().unwrap();

                private.push(".ssh");
                private.push("id_rsa");

                Cred::ssh_key(username.unwrap(), None, &private, None)
            });

            options.remote_callbacks(callbacks);

            repo.fetch_options(options);

            Cow::Owned(url.to_string())
        }
        RepositoryUri::Url(url) => Cow::Borrowed(url.as_str()),
    };

    // Clone.
    let repo = repo
        .clone(url.as_ref(), dest.as_ref())
        .map_err(|e| CloneError::CloneFailed(e))?;

    repo.cleanup_state().unwrap();

    Ok(())
}

// https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs
pub fn pull<P: AsRef<Path>>(
    path: P,
    options: &HashMap<String, serde_yaml::Value>,
) -> Result<(), PullError> {
    let repo = Repository::open(path).map_err(|e| PullError::RepositoryOpenFailed(e))?;

    // Find current branch on the local.
    let reference = repo.head().unwrap();
    let mut local = reference.resolve().unwrap();

    if !local.is_branch() {
        panic!("locenv currently does not support pulling on a repository with HEAD other than a branch")
    }

    match get_branch_to_clone(options) {
        Ok(branch) => {
            if local.shorthand().unwrap() != branch {
                panic!("locenv currently does not support pulling from a different branch")
            }
        }
        Err(e) => match e {
            GetBranchToCloneError::NoOption => {}
            GetBranchToCloneError::InvalidValue(key) => return Err(PullError::InvalidOption(key)),
        },
    }

    // Fetch origin.
    let mut remote = repo
        .find_remote("origin")
        .map_err(|e| PullError::FindOriginFailed(e))?;

    remote
        .fetch(&[] as &[String], None, None)
        .map_err(|e| PullError::FetchOriginFailed(e))?;

    // Find a commit on the remote to merge.
    let reference = repo.find_reference("FETCH_HEAD").unwrap();
    let latest = repo.reference_to_annotated_commit(&reference).unwrap();

    drop(reference);

    // Merge.
    let (ma, _) = repo.merge_analysis(&[&latest]).unwrap();

    if ma.is_fast_forward() {
        let name: String = local.name().unwrap().into();
        let id = latest.id();
        let msg = format!("Fast-Forward: Setting {} to id: {}", name, id);
        let mut options = git2::build::CheckoutBuilder::default();

        options.force();
        options.remove_ignored(true);

        local.set_target(id, &msg).unwrap();
        repo.set_head(&name).unwrap();
        repo.checkout_head(Some(&mut options)).unwrap();
    } else if !ma.is_up_to_date() {
        // This should never happen for now.
        panic!("Cannot fast-forward for some reason")
    }

    // Clean up.
    repo.cleanup_state().unwrap();

    Ok(())
}

#[derive(Debug)]
pub enum CloneError {
    InvalidOption(&'static str),
    CloneFailed(git2::Error),
}

#[derive(Debug)]
pub enum PullError {
    RepositoryOpenFailed(git2::Error),
    FindOriginFailed(git2::Error),
    FetchOriginFailed(git2::Error),
    InvalidOption(&'static str),
}

fn get_branch_to_clone(
    options: &HashMap<String, serde_yaml::Value>,
) -> Result<&str, GetBranchToCloneError> {
    let key = "branch";
    let value = options.get(key).ok_or(GetBranchToCloneError::NoOption)?;

    match value {
        serde_yaml::Value::String(v) => Ok(v),
        _ => Err(GetBranchToCloneError::InvalidValue(key)),
    }
}

enum GetBranchToCloneError {
    NoOption,
    InvalidValue(&'static str),
}
