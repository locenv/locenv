use super::command::Command;
use clap::{value_parser, Arg};
use context::Context;
use module::{Module, PackageId};
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
    let install = clap::Command::new("install").about("Install a module").arg(
        Arg::new("id")
            .help("Package identifier (e.g. github:locenv/autoconf)")
            .required(true)
            .value_parser(value_parser!(PackageId)),
    );
    let update = clap::Command::new("update").about("Update installed module(s)");

    clap::Command::new(name)
        .about("Manage the modules")
        .subcommand_required(true)
        .subcommand(install)
        .subcommand(update)
}

fn run(context: &Context, args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
    if let Some(i) = args.subcommand_matches("install") {
        let id: &PackageId = i.get_one("id").unwrap();

        Module::install(context, id)?;
    } else {
        panic!("Sub-command not implemented")
    }

    Ok(())
}
