pub use self::package::PackageId;

use self::instance::{Instance, LoadError};
use self::metadata::MetadataManager;
use self::package::{InstallationScope, PackageReader, Registry};
use config::module::ModuleDefinition;
use context::modules::module::ModuleContent;
use context::Context;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::create_dir_all;
use std::path::Path;
use std::path::PathBuf;
use zip::ZipArchive;

pub mod instance;
pub mod metadata;

mod github;
mod package;

pub struct Module<'context, 'name> {
    path: PathBuf,
    definition: ModuleDefinition,
    metadata: MetadataManager<'context, 'name>,
}

#[derive(Debug)]
pub enum FindError {
    LoadDefinitionFailed(PathBuf, config::FromFileError),
}

#[derive(Debug)]
pub enum InstallError {
    InvalidIdentifier,
    GetPackageFailed(Box<dyn Error>),
    AlreadyInstalled,
}

// Module

impl<'context, 'name> Module<'context, 'name> {
    pub fn find(context: &'context Context, name: &'name str) -> Result<Self, FindError> {
        let context = context.modules().by_name(name);

        // Load module definition.
        let path = context.definition();
        let definition = match ModuleDefinition::from_file(&path) {
            Ok(r) => r,
            Err(e) => return Err(FindError::LoadDefinitionFailed(path, e)),
        };

        Ok(Module {
            path: context.path(),
            definition,
            metadata: MetadataManager::new(context.metadata()),
        })
    }

    pub fn install(context: &'context Context, id: &PackageId) -> Result<Self, InstallError> {
        // Download module package.
        let package = match id.registry() {
            Registry::GitHub => match github::get_latest_package(id.name()) {
                Ok(r) => r,
                Err(e) => match e {
                    github::Error::InvalidIdentifier => {
                        return Err(InstallError::InvalidIdentifier)
                    }
                    e => return Err(InstallError::GetPackageFailed(e.into())),
                },
            },
        };

        // Extract the package.
        let content = tempfile::tempdir().unwrap();
        let mut extractor = ZipArchive::new(package).unwrap();

        extractor.extract(&content).unwrap();

        // Read definition.
        let path = PackageReader::new(content.path()).definition();
        let definition = ModuleDefinition::from_file(&path).unwrap();

        // Check if installation can be proceed.
        let context = context.modules().by_owned(&definition.name);
        let path = context.path();

        if path.exists() {
            return Err(InstallError::AlreadyInstalled);
        }

        // Install.
        let mut scope = InstallationScope::new(&path);
        let mut options = fs_extra::dir::CopyOptions::new();

        options.copy_inside = true;
        options.content_only = true;

        create_dir_all(&path).unwrap();
        fs_extra::dir::copy(&content, &path, &options).unwrap();

        // Write metadata.
        let metadata = MetadataManager::new(context.metadata());

        metadata.write_registry(&id);

        // Mark installation success.
        scope.success();
        drop(scope);

        Ok(Module {
            path,
            definition,
            metadata,
        })
    }

    pub fn load(&self) -> Result<Instance, LoadError> {
        Instance::load(self.path.join(&self.definition.file))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

// FindError

impl Error for FindError {}

impl Display for FindError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::LoadDefinitionFailed(p, e) => {
                write!(f, "Failed to load {}: {}", p.display(), e)
            }
        }
    }
}

// InstallError

impl Error for InstallError {}

impl Display for InstallError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            InstallError::InvalidIdentifier => write!(f, "The package identifer is not valid"),
            InstallError::GetPackageFailed(e) => write!(f, "Failed to get the package: {}", e),
            InstallError::AlreadyInstalled => write!(f, "The module is already installed"),
        }
    }
}
