use super::config::Repository;

pub struct UpdateError {
}

pub async fn update(_: &Repository) -> Result<(), UpdateError> {
    Ok(())
}

impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown error")
    }
}
