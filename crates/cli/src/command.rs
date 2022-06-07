use context::Context;
use std::error::Error;

pub struct Command {
    pub name: &'static str,
    pub specs: fn(name: &str) -> clap::Command<'static>,
    pub manager_running: Option<bool>,
    pub run: fn(context: &Context, args: &clap::ArgMatches) -> Result<(), Box<dyn Error>>,
}
