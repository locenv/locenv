use super::command::Command;
use clap::{value_parser, Arg};
use context::Context;
use module::{Module, PackageId};
use std::borrow::Cow;
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
            .help("Package identifier (e.g. github:locenv/mod-autoconf)")
            .required(true)
            .value_parser(value_parser!(PackageId)),
    );
    let update = clap::Command::new("update")
        .about("Update installed module")
        .arg(
            Arg::new("name")
                .help("Name of the module to update")
                .required(true),
        );

    clap::Command::new(name)
        .about("Manage the modules")
        .subcommand_required(true)
        .subcommand(install)
        .subcommand(update)
}

fn run(context: &Context, args: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
    if let Some(args) = args.subcommand_matches("install") {
        let id: &PackageId = args.get_one("id").unwrap();

        Module::install(context, id)?;
    } else if let Some(args) = args.subcommand_matches("update") {
        let name = args.get_one::<String>("name").unwrap();

        Module::update(context, Cow::Borrowed(&name))?;
    } else {
        panic!("Sub-command not implemented")
    }

    Ok(())
}
