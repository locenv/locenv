use crate::package::PackageId;
use context::data::ModuleMetadata;
use dirtree::TextFile;
use dirtree_macros::Directory;
use std::marker::PhantomData;
use std::path::PathBuf;

#[derive(Directory)]
pub struct MetadataManager<'context, 'module> {
    context: ModuleMetadata<'context, 'module>,

    #[file(pub)]
    registry: PhantomData<TextFile<PackageId>>,
}

impl<'context, 'module> MetadataManager<'context, 'module> {
    pub(super) fn new(context: ModuleMetadata<'context, 'module>) -> Self {
        MetadataManager {
            context,
            registry: PhantomData,
        }
    }

    pub fn directory_name(&self) -> &'static str {
        self.context.name()
    }

    pub fn path(&self) -> PathBuf {
        self.context.path()
    }
}
