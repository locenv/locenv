use crate::context::Context;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;

pub struct Command<'run> {
    pub name: &'static str,
    pub specs: fn(name: &str) -> clap::Command<'static>,
    pub run: fn(
        context: &'run Context,
        args: &'run clap::ArgMatches,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + 'run>>,
}
