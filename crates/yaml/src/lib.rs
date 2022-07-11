use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::path::Path;

pub fn load_file<P, T>(path: P) -> Result<T, FileError>
where
    P: AsRef<Path>,
    T: serde::de::DeserializeOwned,
{
    let file = File::open(&path).map_err(|e| FileError::OpenFailed(e))?;
    let data = serde_yaml::from_reader(file).map_err(|e| FileError::ParseFailed(e.into()))?;

    Ok(data)
}

#[derive(Debug)]
pub enum FileError {
    OpenFailed(std::io::Error),
    ParseFailed(Box<dyn Error>),
}

impl Error for FileError {}

impl Display for FileError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            FileError::OpenFailed(e) => write!(f, "{}", e),
            FileError::ParseFailed(e) => write!(f, "{}", e),
        }
    }
}
