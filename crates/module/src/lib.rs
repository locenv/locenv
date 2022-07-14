pub use self::definition::ModuleDefinition;
pub use self::package::PackageId;

use self::metadata::MetadataManager;
use self::package::{InstallationScope, PackageReader, Registry};
use context::data::ModuleDirectory;
use context::Context;
use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::create_dir_all;
use std::hash::Hash;
use std::io::{Read, Seek};
use std::path::Path;
use std::path::PathBuf;
use tempfile::TempDir;
use zip::ZipArchive;

pub mod definition;
pub mod metadata;

mod github;
mod package;

#[allow(dead_code)]
pub struct Module<'context, 'name> {
    definition: ModuleDefinition,
    path: PathBuf,
    metadata: MetadataManager<'context, 'name>,
}

#[derive(Debug)]
pub enum FindError {
    NotInstalled(PathBuf),
    LoadDefinitionFailed(PathBuf, yaml::FileError),
}

impl<'context, 'name> Module<'context, 'name> {
    pub fn find(context: &'context Context, name: Cow<'name, str>) -> Result<Self, FindError> {
        let context = context.data().module().by_name(name);
        let path = context.path();

        // Check if module directory exists.
        if !path.exists() {
            return Err(FindError::NotInstalled(path));
        }

        // Load module definition.
        let file = context.definition();
        let definition: ModuleDefinition =
            yaml::load_file(&file).map_err(|e| FindError::LoadDefinitionFailed(file, e))?;

        Ok(Module {
            definition,
            path,
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

        let (content, definition) = Self::extract_package(package);

        // Check if installation can be proceed.
        let context = context
            .data()
            .module()
            .by_name(Cow::Owned(definition.name.clone()));
        let path = context.path();

        if path.exists() {
            return Err(InstallError::AlreadyInstalled(definition.name.clone()));
        }

        // Install.
        let mut scope = InstallationScope::new(&path);

        create_dir_all(&path).unwrap();
        Self::install_package(&content, &path);

        // Write metadata.
        let metadata = MetadataManager::new(context.metadata());

        metadata.registry().write(&id).unwrap();

        // Mark installation success.
        scope.success();
        drop(scope);

        Ok(Module {
            path,
            definition,
            metadata,
        })
    }

    pub fn update(context: &'context Context, name: Cow<'name, str>) -> Result<Self, UpdateError> {
        // Check if module installed.
        let context = context.data().module().by_name(name);
        let path = context.path();

        if !path.exists() {
            return Err(UpdateError::NotInstalled);
        }

        let local: ModuleDefinition = yaml::load_file(context.definition()).unwrap();

        // Get registry.
        let metadata = MetadataManager::new(context.metadata());
        let id = metadata.registry().read().unwrap();

        // Download latest package.
        let package = match id.registry() {
            Registry::GitHub => match github::get_latest_package(id.name()) {
                Ok(r) => r,
                Err(e) => return Err(UpdateError::GetPackageFailed(e.into())),
            },
        };

        let (content, remote) = Self::extract_package(package);

        // Check if the installed version already up todate.
        if local.version >= remote.version {
            return Err(UpdateError::AlreadyLatest);
        }

        // Update.
        let mut scope = InstallationScope::new(&path);

        for file in std::fs::read_dir(&path).unwrap().map(|i| i.unwrap()) {
            if file.file_name() == metadata.directory_name() {
                continue;
            }

            if file.file_type().unwrap().is_dir() {
                std::fs::remove_dir_all(file.path()).unwrap();
            } else {
                std::fs::remove_file(file.path()).unwrap();
            }
        }

        Self::install_package(&content, &path);

        // Mark update success.
        scope.success();
        drop(scope);

        Ok(Module {
            path,
            definition: remote,
            metadata,
        })
    }

    pub fn definition(&self) -> &ModuleDefinition {
        &self.definition
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn extract_package<F: Read + Seek>(package: F) -> (TempDir, ModuleDefinition) {
        // Extract.
        let content = tempfile::tempdir().unwrap();
        let mut extractor = ZipArchive::new(package).unwrap();

        extractor.extract(&content).unwrap();

        // Read definition.
        let path = PackageReader::new(content.path()).definition();
        let definition: ModuleDefinition = yaml::load_file(&path).unwrap();

        (content, definition)
    }

    fn install_package<C: AsRef<Path>, D: AsRef<Path>>(content: C, destination: D) {
        let mut options = fs_extra::dir::CopyOptions::new();

        options.copy_inside = true;
        options.content_only = true;

        fs_extra::dir::copy(content, destination, &options).unwrap();
    }
}

impl<'context, 'name> Eq for Module<'context, 'name> {}

impl<'context, 'name> Hash for Module<'context, 'name> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.definition.name.hash(state);
    }
}

impl<'context, 'name> PartialEq for Module<'context, 'name> {
    fn eq(&self, other: &Self) -> bool {
        self.definition.name == other.definition.name
    }
}

impl Error for FindError {}

impl Display for FindError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            FindError::NotInstalled(_) => write!(f, "The module is not installed"),
            FindError::LoadDefinitionFailed(p, e) => {
                write!(f, "Failed to load {}: {}", p.display(), e)
            }
        }
    }
}

pub enum InstallError {
    InvalidIdentifier,
    GetPackageFailed(Box<dyn Error>),
    AlreadyInstalled(String),
}

pub enum UpdateError {
    NotInstalled,
    GetPackageFailed(Box<dyn Error>),
    AlreadyLatest,
}
