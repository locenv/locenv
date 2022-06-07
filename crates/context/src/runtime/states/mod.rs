use self::state::State;
use super::Runtime;
use std::borrow::Cow;
use std::path::PathBuf;

pub mod state;

pub struct States<'context> {
    runtime: Runtime<'context>,
    name: &'static str,
}

impl<'context> States<'context> {
    pub(super) fn new(runtime: Runtime<'context>, name: &'static str) -> Self {
        States { runtime, name }
    }

    pub fn by_name<'name>(self, name: &'name str) -> State<'context, 'name> {
        State::new(self, Cow::Borrowed(name))
    }

    pub fn path(&self) -> PathBuf {
        self.runtime.path().join(self.name)
    }
}
