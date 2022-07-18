use crate::header::Header;
use crate::status;
use std::fmt::{Debug, Display};

pub trait Handler {
    type Output;
    type Err: std::error::Error + Debug + Display;

    fn process_status(&mut self, line: &status::Line) -> Result<(), Self::Err>;
    fn process_header(&mut self, name: &Header, value: &str) -> Result<(), Self::Err>;
    fn begin_body(&mut self) -> Result<(), Self::Err>;
    fn process_body(&mut self, chunk: &[u8]) -> Result<(), Self::Err>;
    fn take_output(&mut self) -> Result<Self::Output, Self::Err>;
}
