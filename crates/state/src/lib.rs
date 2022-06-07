use context::runtime::states::state::State;
use std::fs::{create_dir_all, remove_dir_all};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct StateManager<'context, 'name> {
    context: State<'context, 'name>,
}

impl<'context, 'name> StateManager<'context, 'name> {
    pub fn new(context: State<'context, 'name>) -> Self {
        StateManager { context }
    }

    pub fn read_built_time(&self) -> Option<SystemTime> {
        // Read file.
        let path = self.built_time_path();
        let data = match std::fs::read_to_string(&path) {
            Ok(r) => r,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    return None;
                } else {
                    panic!("Failed to read {}: {}", path.display(), e);
                }
            }
        };

        // Parse data.
        let unix: u64 = data.trim().parse().unwrap();
        let time = UNIX_EPOCH + Duration::from_secs(unix);

        Some(time)
    }

    pub fn write_built_time(&self, time: &SystemTime) {
        let unix = time.duration_since(UNIX_EPOCH).unwrap();
        let mut data = unix.as_secs().to_string();

        data.push('\n');

        std::fs::write(self.built_time_path(), &data).unwrap();
    }

    pub fn clear(&self) {
        let path = self.context.path();

        if let Err(e) = remove_dir_all(&path) {
            if e.kind() != std::io::ErrorKind::NotFound {
                panic!("Failed to remove {}: {}", path.display(), e);
            }
        }
    }

    fn built_time_path(&self) -> PathBuf {
        let mut p = self.ensure_path();
        p.push("built-time");
        p
    }

    fn ensure_path(&self) -> PathBuf {
        let p = self.context.path();

        if !p.exists() {
            create_dir_all(&p).unwrap();
        }

        p
    }
}
