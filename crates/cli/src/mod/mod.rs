use super::command::Command;
use context::Context;
use std::error::Error;

pub fn command() -> Command {
    Command {
        name: "mod",
        specs,
        manager_running: Some(false),
        run,
    }
}

fn specs(name: &str) -> clap::Command<'static> {
    let install = clap::Command::new("install").about("Install a module");
    let update = clap::Command::new("update").about("Update installed module(s)");

    clap::Command::new(name)
        .about("Manage the modules")
        .subcommand_required(true)
        .subcommand(install)
        .subcommand(update)
}

fn run(_: &Context, _: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
    Ok(())
}
