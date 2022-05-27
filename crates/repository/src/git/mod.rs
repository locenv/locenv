use config::RepositoryUri;
use self::errors::{
    AnnotatedCommitFromReferenceError, CheckoutError, CloneError, FetchError, FindReferenceError,
    FindRemoteError, MergeAnalysisError, OpenRepositoryError, SetHeadError, SetReferenceError,
};
use std::error::Error;
use std::path::Path;

mod errors;

pub async fn download<P: AsRef<Path>>(path: P, uri: &RepositoryUri) -> Result<(), Box<dyn Error>> {
    // Setup repo builder.
    let mut rb = git2::build::RepoBuilder::new();
    let url: String;

    match uri {
        RepositoryUri::Scp(v) => {
            // Setup credential callback.
            let mut cb = git2::RemoteCallbacks::new();

            cb.credentials(|_, username, _| {
                let mut private = dirs::home_dir().unwrap();

                private.push(".ssh");
                private.push("id_rsa");

                git2::Cred::ssh_key(username.unwrap(), None, &private, None)
            });

            // Setup options.
            let mut opt = git2::FetchOptions::new();

            opt.remote_callbacks(cb);
            rb.fetch_options(opt);

            // Get URL.
            url = v.into_string();
        }
        RepositoryUri::Url(v) => {
            url = v.as_str().into();
        }
    };

    // Clone.
    let repo = match rb.clone(&url, path.as_ref()) {
        Ok(r) => r,
        Err(e) => return Err(CloneError::new(url, e).into()),
    };

    repo.cleanup_state().unwrap();

    Ok(())
}

// https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs
pub async fn update<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn Error>> {
    // Open repository.
    let repo = match git2::Repository::open(&path) {
        Ok(r) => r,
        Err(e) => return Err(OpenRepositoryError::new(path.as_ref(), e).into()),
    };

    // Get origin.
    let mut remote = match repo.find_remote("origin") {
        Ok(r) => r,
        Err(e) => return Err(FindRemoteError::new(path.as_ref(), "origin", e).into()),
    };

    // Fetch origin.
    if let Err(e) = remote.fetch(&[] as &[String], None, None) {
        return Err(FetchError::new(path.as_ref(), remote.name().unwrap(), e).into());
    }

    drop(remote);

    // Find commit on remote to merge.
    let reference = match repo.find_reference("FETCH_HEAD") {
        Ok(r) => r,
        Err(e) => return Err(FindReferenceError::new(path.as_ref(), "FETCH_HEAD", e).into()),
    };

    let latest = match repo.reference_to_annotated_commit(&reference) {
        Ok(r) => r,
        Err(e) => {
            return Err(AnnotatedCommitFromReferenceError::new(
                path.as_ref(),
                reference.name().unwrap(),
                e,
            )
            .into())
        }
    };

    drop(reference);

    // Determine how to merge.
    let (ma, _) = match repo.merge_analysis(&[&latest]) {
        Ok(r) => r,
        Err(e) => {
            return Err(MergeAnalysisError::new(path.as_ref(), latest.refname().unwrap(), e).into())
        }
    };

    // Do merge.
    let mut target = match repo.find_reference("refs/heads/main") {
        Ok(r) => r,
        Err(e) => return Err(FindReferenceError::new(path.as_ref(), "refs/heads/main", e).into()),
    };

    if ma.is_fast_forward() {
        let name = target.name().unwrap().to_owned();
        let id = latest.id();
        let msg = format!("Fast-Forward: Setting {} to id: {}", name, id);

        if let Err(e) = target.set_target(id, &msg) {
            return Err(SetReferenceError::new(path.as_ref(), name, id, e).into());
        }

        if let Err(e) = repo.set_head(&name) {
            return Err(SetHeadError::new(path.as_ref(), name, e).into());
        }

        if let Err(e) = repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force())) {
            return Err(CheckoutError::new(path.as_ref(), "HEAD", e).into());
        }
    }

    // Clean up.
    drop(latest);
    drop(target);

    repo.cleanup_state().unwrap();

    Ok(())
}
