use crate::package::PackageId;
use context::modules::module::metadata::Metadata;
use fmap_macros::Directory;
use std::path::PathBuf;

#[derive(Directory)]
pub struct MetadataManager<'context, 'module> {
    context: Metadata<'context, 'module>,

    #[file]
    registry: fmap::Text,
}

impl<'context, 'module> MetadataManager<'context, 'module> {
    pub(super) fn new(context: Metadata<'context, 'module>) -> Self {
        MetadataManager {
            context,
            registry: fmap::Text,
        }
    }

    pub fn write_registry(&self, package: &PackageId) {
        let mut data = package.to_string();

        data.push('\n');

        self.registry().write(&data).unwrap();
    }

    pub fn read_registry(&self) -> Option<PackageId> {
        self.registry().read().map(|v| v.trim().parse().unwrap())
    }

    /// Gets the name of directory all metadata is stored. This returns only a directory name, not a path.
    pub fn directory(&self) -> &str {
        self.context.name()
    }
}

impl<'context, 'module> fmap::Parent for MetadataManager<'context, 'module> {
    fn path(&self) -> PathBuf {
        self.context.path()
    }
}
