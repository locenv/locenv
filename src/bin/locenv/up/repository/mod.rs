use super::config::{Repository, RepositoryType};
use crate::context::Context;
use std::error::Error;

mod git;

pub async fn update<N: AsRef<str>>(ctx: &Context, name: N, repo: &Repository) -> Result<(), Box<dyn Error>> {
    match repo.r#type {
        RepositoryType::Git => git::update(ctx, name, &repo.uri).await
    }
}
