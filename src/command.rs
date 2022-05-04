use std::future::Future;
use std::pin::Pin;

pub struct Command<'args> {
    pub name: &'static str,
    pub specs: fn(name: &str) -> clap::Command<'static>,
    pub run: fn(args: &'args clap::ArgMatches) -> Pin<Box<dyn Future<Output = Result<(), String>> + 'args>>
}
