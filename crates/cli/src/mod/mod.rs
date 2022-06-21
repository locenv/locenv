use super::command::Command;
use clap::{value_parser, Arg};
use context::Context;
use std::error::Error;

mod install;

pub fn command() -> Command {
    Command {
        name: "mod",
        specs,
        manager_running: Some(false),
        run,
    }
}

fn specs(name: &str) -> clap::Command<'static> {
    let install = clap::Command::new("install").about("Install a module").arg(
        Arg::new("spec")
            .help("Module specifications (e.g. gh:locenv/autoconf)")
            .required(true)
            .value_parser(value_parser!(self::install::Spec)),
    );
    let update = clap::Command::new("update").about("Update installed module(s)");

    clap::Command::new(name)
        .about("Manage the modules")
        .subcommand_required(true)
        .subcommand(install)
        .subcommand(update)
}

fn run(_: &Context, args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
    if let Some(i) = args.subcommand_matches("install") {
        let spec: &self::install::Spec = i.get_one("spec").unwrap();

        self::install::run(spec)
    } else {
        panic!("Sub-command not implemented")
    }
}
