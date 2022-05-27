use self::errors::{ConfigOpenError, ConfigParseError};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::iter::IntoIterator;
use std::path::Path;

mod errors;

#[derive(Deserialize)]
pub struct Services(HashMap<String, Service>);

#[derive(Deserialize)]
pub struct Service {
    pub repository: Repository,
}

#[derive(Deserialize)]
pub struct Repository {
    pub r#type: RepositoryType,
    pub uri: RepositoryUri,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryType {
    Git,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum RepositoryUri {
    Scp(super::scp::Url),
    Url(url::Url),
}

impl Services {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = match File::open(&path) {
            Ok(r) => r,
            Err(e) => return Err(ConfigOpenError::new(path.as_ref(), e).into()),
        };

        let config = match Services::from_reader(file) {
            Ok(r) => r,
            Err(e) => return Err(ConfigParseError::new(path.as_ref(), e).into()),
        };

        Ok(config)
    }

    pub fn from_reader<R: Read>(reader: R) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_reader(reader)
    }
}

impl<'a> IntoIterator for &'a Services {
    type Item = (&'a String, &'a Service);
    type IntoIter = std::collections::hash_map::Iter<'a, String, Service>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Seek, Write};
    use tempfile::tempfile;

    #[test]
    fn test_from_reader() {
        // Generate configurations.
        let mut file = tempfile().unwrap();
        let yml = b"postgres:
  repository:
    type: git
    uri: git@github.com:example/repository.git
redis:
  repository:
    type: git
    uri: https://github.com/example/repository.git";

        file.write_all(yml).unwrap();
        file.rewind().unwrap();

        // Load.
        let result = Services::from_reader(file).unwrap();

        // Asserts.
        assert!(result.0.contains_key("postgres"));
        assert!(result.0.contains_key("redis"));

        let postgres = result.0.get("postgres").unwrap();
        let redis = result.0.get("redis").unwrap();

        assert_eq!(postgres.repository.r#type, RepositoryType::Git);
        assert_eq!(
            postgres.repository.uri,
            RepositoryUri::Scp(crate::scp::Url {
                user: Some(String::from("git")),
                host: String::from("github.com"),
                path: Some(String::from("example/repository.git"))
            })
        );
        assert_eq!(redis.repository.r#type, RepositoryType::Git);
        assert_eq!(
            redis.repository.uri,
            RepositoryUri::Url(
                url::Url::parse("https://github.com/example/repository.git").unwrap()
            )
        );
    }
}
