use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub struct IncorrectManagerState {
    expected: bool
}

// IncorrectManagerState

impl IncorrectManagerState {
    pub fn new(expected: bool) -> Self {
        IncorrectManagerState { expected }
    }
}

impl Error for IncorrectManagerState {}

impl Display for IncorrectManagerState {
    fn fmt(&self, f: &mut Formatter) -> Result {
        if self.expected {
            f.write_str("The services is not running")
        } else {
            f.write_str("The services is already running")
        }
    }
}
