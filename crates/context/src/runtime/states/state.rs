use super::States;
use std::borrow::Cow;
use std::path::PathBuf;

pub struct State<'context, 'name> {
    parent: States<'context>,
    name: Cow<'name, str>,
}

impl<'context, 'name> State<'context, 'name> {
    pub(super) fn new(parent: States<'context>, name: Cow<'name, str>) -> Self {
        State { parent, name }
    }

    pub fn path(&self) -> PathBuf {
        let mut p = self.parent.path();
        p.push(self.name.as_ref());
        p
    }
}
