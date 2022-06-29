use crate::package::PackageId;
use context::modules::module::metadata::Metadata;
use std::fs::create_dir_all;
use std::path::PathBuf;

pub struct MetadataManager<'context, 'module> {
    context: Metadata<'context, 'module>,
}

impl<'context, 'module> MetadataManager<'context, 'module> {
    pub(super) fn new(context: Metadata<'context, 'module>) -> Self {
        MetadataManager { context }
    }

    pub fn write_registry(&self, package: &PackageId) {
        let mut data = package.to_string();

        data.push('\n');

        std::fs::write(self.registry_path(), &data).unwrap();
    }

    fn registry_path(&self) -> PathBuf {
        let mut path = self.ensure_path();
        path.push("registry");
        path
    }

    fn ensure_path(&self) -> PathBuf {
        let path = self.context.path();

        if !path.exists() {
            create_dir_all(&path).unwrap();
        }

        path
    }
}
