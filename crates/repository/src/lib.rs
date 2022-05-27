use config::{Repository, RepositoryType};
use std::error::Error;
use std::path::Path;

mod git;

pub async fn download<P: AsRef<Path>>(path: P, repo: &Repository) -> Result<(), Box<dyn Error>> {
    match repo.r#type {
        RepositoryType::Git => git::download(path, &repo.uri).await,
    }
}

pub async fn update<P: AsRef<Path>>(path: P, repo: &Repository) -> Result<(), Box<dyn Error>> {
    match repo.r#type {
        RepositoryType::Git => git::update(path).await,
    }
}
