use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs::create_dir_all;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn ensure_path<'path>(path: &'path PathBuf) -> std::io::Result<&'path PathBuf> {
    if !path.exists() {
        create_dir_all(path)?;
    }

    Ok(path)
}

/// Represents a file to store [`SystemTime`].
pub struct TimestampFile {
    parent: PathBuf,
    name: &'static str,
}

impl TimestampFile {
    pub fn new(parent: PathBuf, name: &'static str) -> Self {
        Self { parent, name }
    }

    pub fn write(&self, value: &SystemTime) -> Result<(), TimestampFileError> {
        let path = ensure_path(&self.parent)
            .map_err(|e| TimestampFileError::CreateParentFailed(e))?
            .join(self.name);
        let unix = value
            .duration_since(UNIX_EPOCH)
            .map_err(|_| TimestampFileError::NotEpoch)?;
        let mut data = unix.as_secs().to_string();

        data.push('\n');

        std::fs::write(&path, &data).map_err(|e| TimestampFileError::WriteFailed(e))
    }

    pub fn read(&self) -> Result<SystemTime, TimestampFileError> {
        let path = self.path();
        let data = std::fs::read_to_string(&path).map_err(|e| TimestampFileError::ReadFailed(e))?;
        let unix: u64 = data
            .trim()
            .parse()
            .map_err(|e| TimestampFileError::ParseFailed(e))?;

        Ok(UNIX_EPOCH + Duration::from_secs(unix))
    }

    pub fn path(&self) -> PathBuf {
        self.parent.join(self.name)
    }
}

#[derive(Debug)]
pub enum TimestampFileError {
    CreateParentFailed(std::io::Error),
    NotEpoch,
    WriteFailed(std::io::Error),
    ReadFailed(std::io::Error),
    ParseFailed(<u64 as FromStr>::Err),
}

impl Error for TimestampFileError {}

impl Display for TimestampFileError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            TimestampFileError::CreateParentFailed(e) => {
                write!(f, "Failed to create a parent directory: {}", e)
            }
            TimestampFileError::NotEpoch => write!(f, "The value is not a valid Unix time"),
            TimestampFileError::WriteFailed(e) => write!(f, "Failed to write file: {}", e),
            TimestampFileError::ReadFailed(e) => write!(f, "Failed to read file: {}", e),
            TimestampFileError::ParseFailed(e) => write!(f, "Failed to parse file: {}", e),
        }
    }
}

/// Represents a file to store data as text.
pub struct TextFile<D>
where
    D: ToString + FromStr + Debug,
    <D as FromStr>::Err: Display + Debug,
{
    parent: PathBuf,
    name: &'static str,
    phantom: PhantomData<D>,
}

impl<D> TextFile<D>
where
    D: ToString + FromStr + Debug,
    <D as FromStr>::Err: Display + Debug,
{
    pub fn new(parent: PathBuf, name: &'static str) -> Self {
        Self {
            parent,
            name,
            phantom: PhantomData,
        }
    }

    pub fn write(&self, value: &D) -> std::io::Result<()> {
        let path = ensure_path(&self.parent)?.join(self.name);
        let mut data = value.to_string();

        data.push('\n');

        std::fs::write(path, data)
    }

    pub fn read(&self) -> Result<D, TextFileError<D>> {
        let path = self.path();
        let data = std::fs::read_to_string(&path).map_err(|e| TextFileError::ReadFailed(e))?;

        data.trim()
            .parse()
            .map_err(|e| TextFileError::ParseFailed(e))
    }

    pub fn path(&self) -> PathBuf {
        self.parent.join(self.name)
    }
}

#[derive(Debug)]
pub enum TextFileError<D>
where
    D: FromStr + Debug,
    <D as FromStr>::Err: Display + Debug,
{
    ReadFailed(std::io::Error),
    ParseFailed(D::Err),
}

impl<D> Error for TextFileError<D>
where
    D: FromStr + Debug,
    <D as FromStr>::Err: Display + Debug,
{
}

impl<D> Display for TextFileError<D>
where
    D: FromStr + Debug,
    <D as FromStr>::Err: Display + Debug,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            TextFileError::ReadFailed(e) => write!(f, "Failed to read file: {}", e),
            TextFileError::ParseFailed(e) => write!(f, "Failed to parse file: {}", e),
        }
    }
}

#[derive(Debug)]
pub enum DirectoryError {
    CreateFailed(std::io::Error),
}
