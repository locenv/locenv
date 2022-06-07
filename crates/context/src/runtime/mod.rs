use self::datas::Datas;
use self::manager::Manager;
use self::repositories::Repositories;
use self::states::States;
use std::path::Path;

pub mod datas;
pub mod manager;
pub mod repositories;
pub mod states;

pub struct Runtime<'context> {
    path: &'context Path,
}

impl<'context> Runtime<'context> {
    pub(super) fn new(path: &'context Path) -> Self {
        Runtime { path }
    }

    pub fn repositories(self) -> Repositories<'context> {
        Repositories::new(self, "repository")
    }

    pub fn datas(self) -> Datas<'context> {
        Datas::new(self, "data")
    }

    pub fn states(self) -> States<'context> {
        States::new(self, "state")
    }

    pub fn manager(self) -> Manager<'context> {
        Manager::new(self, "manager")
    }

    pub fn path(&self) -> &Path {
        self.path
    }
}
