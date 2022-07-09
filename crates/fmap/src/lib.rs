use std::fs::create_dir_all;
use std::path::PathBuf;

pub trait Parent {
    fn path(&self) -> PathBuf;

    fn ensure(&self) -> std::io::Result<PathBuf> {
        let path = self.path();

        if !path.exists() {
            create_dir_all(&path)?;
        }

        Ok(path)
    }
}

pub struct Text;

#[allow(dead_code)]
pub struct TextFile<'parent, P: Parent> {
    parent: &'parent P,
    field: &'parent Text,
    name: &'static str,
}

impl<'parent, P: Parent> TextFile<'parent, P> {
    pub fn new(parent: &'parent P, field: &'parent Text, name: &'static str) -> Self {
        TextFile {
            parent,
            field,
            name,
        }
    }

    pub fn write(&self, value: &str) -> std::io::Result<()> {
        let path = self.parent.ensure()?.join(self.name);

        std::fs::write(path, value)
    }

    pub fn read(&self) -> Option<String> {
        let path = self.parent.path().join(self.name);

        match std::fs::read_to_string(&path) {
            Ok(r) => Some(r),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => None,
                _ => panic!("Failed to read {}: {}", path.display(), e),
            },
        }
    }
}
